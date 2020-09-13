pub mod fresnel;

use super::sampling::{
    cosine_sample_hemisphere, uniform_hemisphere_pdf, uniform_sample_hemisphere,
};
use crate::common::spectrum::Spectrum;
use ambassador::{delegatable_trait, Delegate};
use fresnel::{FresnelSpecular, SpecularReflection, SpecularTransmission};

fn cos_theta(w: &na::Vector3<f32>) -> f32 {
    w.z
}

fn cos_2_theta(w: &na::Vector3<f32>) -> f32 {
    w.z * w.z
}

fn abs_cos_theta(w: &na::Vector3<f32>) -> f32 {
    w.z.abs()
}

fn same_hemisphere(w: &na::Vector3<f32>, wp: &na::Vector3<f32>) -> bool {
    w.z * wp.z > 0.0
}

bitflags! {
    pub struct BxDFType: u32 {
        const BSDF_REFLECTION = 1 << 0;
        const BSDF_TRANSMISSION = 1 << 1;
        const BSDF_DIFFUSE = 1 << 2;
        const BSDF_GLOSSY = 1 << 3;
        const BSDF_SPECULAR = 1 << 4;
        const BSDF_ALL = Self::BSDF_DIFFUSE.bits | Self::BSDF_GLOSSY.bits | Self::BSDF_SPECULAR.bits | Self::BSDF_REFLECTION.bits |
        Self::BSDF_TRANSMISSION.bits;
    }
}

#[delegatable_trait]
pub trait BxDFInterface {
    fn f(&self, wo: &na::Vector3<f32>, wi: &na::Vector3<f32>) -> Spectrum;
    fn sample_f(
        &self,
        wo: &na::Vector3<f32>,
        wi: &mut na::Vector3<f32>,
        u: &na::Point2<f32>,
        pdf: &mut f32,
        _sampled_type: &mut Option<BxDFType>,
    ) -> Spectrum {
        *wi = cosine_sample_hemisphere(&u);
        if wo.z < 0.0 {
            wi.z *= -1.0;
        }

        *pdf = self.pdf(&wo, &wi);
        self.f(&wo, &wi)
    }
    fn rho(
        &self,
        wo: &na::Vector3<f32>,
        n_samples: usize,
        samples: &[na::Point2<f32>],
    ) -> Spectrum {
        let mut r = Spectrum::new(0.0);

        for i in 0..n_samples {
            let mut wi = glm::zero();
            let mut pdf = 0.0;
            let f = self.sample_f(&wo, &mut wi, &samples[i], &mut pdf, &mut None);

            if pdf > 0.0 {
                r += f * abs_cos_theta(&wi) / pdf;
            }
        }

        r / (n_samples as f32)
    }
    fn rho_no_wo(
        &self,
        n_samples: usize,
        samples_1: &[na::Point2<f32>],
        samples_2: &[na::Point2<f32>],
    ) -> Spectrum {
        let mut r = Spectrum::new(0.0);

        for i in 0..n_samples {
            let mut wi = glm::zero();
            let wo = uniform_sample_hemisphere(&samples_1[i]);
            let mut pdf_i = 0.0;
            let pdf_o = uniform_hemisphere_pdf();
            let f = self.sample_f(&wo, &mut wi, &samples_2[i], &mut pdf_i, &mut None);

            if pdf_i > 0.0 {
                r += f * abs_cos_theta(&wi) * abs_cos_theta(&wo) / (pdf_o * pdf_i);
            }
        }

        r / (std::f32::consts::PI * n_samples as f32)
    }

    fn matches_flags(&self, t: BxDFType) -> bool {
        (self.get_type() & t) == self.get_type()
    }
    fn get_type(&self) -> BxDFType;
    fn pdf(&self, wo: &na::Vector3<f32>, wi: &na::Vector3<f32>) -> f32 {
        if same_hemisphere(&wo, &wi) {
            abs_cos_theta(&wi) * std::f32::consts::FRAC_1_PI
        } else {
            0.0
        }
    }
}

#[derive(Delegate)]
#[delegate(BxDFInterface)]
pub enum BxDF {
    Lambertian(LambertianReflection),
    SpecularReflection(SpecularReflection),
    SpecularTransmission(SpecularTransmission),
    FresnelSpecular(FresnelSpecular),
}

pub struct LambertianReflection {
    r: Spectrum,
}

impl LambertianReflection {
    pub fn new(r: Spectrum) -> Self {
        Self { r }
    }
}

impl BxDFInterface for LambertianReflection {
    fn f(&self, _wo: &na::Vector3<f32>, _wi: &na::Vector3<f32>) -> Spectrum {
        self.r * std::f32::consts::FRAC_1_PI
    }

    fn get_type(&self) -> BxDFType {
        BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
    }

    fn rho(
        &self,
        _wo: &na::Vector3<f32>,
        _n_samples: usize,
        _samples: &[na::Point2<f32>],
    ) -> Spectrum {
        self.r
    }

    fn rho_no_wo(
        &self,
        _n_samples: usize,
        _samples_1: &[na::Point2<f32>],
        _samples_2: &[na::Point2<f32>],
    ) -> Spectrum {
        self.r
    }
}
