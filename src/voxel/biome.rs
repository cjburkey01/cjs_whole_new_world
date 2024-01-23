use enum_iterator::{cardinality, Sequence};
use itertools::iproduct;
use num_enum::{FromPrimitive, IntoPrimitive};
use std::fmt::{Display, Formatter};

pub trait Denormalize: Sequence + FromPrimitive<Primitive = u8> {
    fn denormalize(normalized: f64) -> Self {
        let half_count_f = 0.5 * cardinality::<Self>() as f64;
        let humidity = half_count_f * (normalized.min(1.0).max(0.0) + 1.0);
        <Self as FromPrimitive>::from_primitive(humidity.trunc() as u8)
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Sequence, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum BiomeTemperature {
    #[default]
    AlvarPolar,
    Alpine,
    SubAlpine,
    Montane,
    LowerMontane,
    PreMontane,
}

impl Denormalize for BiomeTemperature {}

impl Display for BiomeTemperature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AlvarPolar => "Polar",
                Self::Alpine => "SubPolar",
                Self::SubAlpine => "Boreal",
                Self::Montane => "Coolville",
                Self::LowerMontane => "Temperate",
                Self::PreMontane => "Tropical",
            }
        )
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Sequence, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum BiomeHumidity {
    #[default]
    SuperArid,
    PerArid,
    Arid,
    SemiArid,
    SubHumid,
    Humid,
    PerHumid,
    SuperHumid,
}

impl Denormalize for BiomeHumidity {}

impl Display for BiomeHumidity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SuperArid => "SuperArid",
                Self::PerArid => "PerArid",
                Self::Arid => "Arid",
                Self::SemiArid => "SemiArid",
                Self::SubHumid => "SubHumid",
                Self::Humid => "Humid",
                Self::PerHumid => "PerHumid",
                Self::SuperHumid => "SuperHumid",
            }
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Biome {
    name: String,
    temperature: BiomeTemperature,
    humidity: BiomeHumidity,
}

#[allow(unused)]
impl Biome {
    pub fn new(name: &str, temperature: BiomeTemperature, humidity: BiomeHumidity) -> Self {
        Self {
            name: name.to_string(),
            temperature,
            humidity,
        }
    }

    pub fn empty_at(temperature: BiomeTemperature, humidity: BiomeHumidity) -> Self {
        Self::new(
            &format!("{temperature} {humidity} Place"),
            temperature,
            humidity,
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn temperature(&self) -> BiomeTemperature {
        self.temperature
    }

    pub fn humidity(&self) -> BiomeHumidity {
        self.humidity
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct BiomeTable {
    biomes: Vec<Biome>,
    temperature_max: u8,
    humidity_max: u8,
}

#[allow(unused)]
impl BiomeTable {
    pub fn new() -> Self {
        let temp_max = cardinality::<BiomeTemperature>();
        let humid_max = cardinality::<BiomeHumidity>();
        let temp_max_b = temp_max as u8;
        let humid_max_b = humid_max as u8;
        let mut biomes = Vec::with_capacity(temp_max * humid_max);
        for (temp, humid) in iproduct!(0..temp_max_b, 0..humid_max_b) {
            biomes.push(Biome::empty_at(temp.into(), humid.into()));
        }
        Self {
            biomes,
            temperature_max: temp_max_b,
            humidity_max: humid_max_b,
        }
    }

    /// Will overwrite existing biomes in this position.
    /// Todo: Keep multiple per index?
    pub fn insert(&mut self, biome: Biome) {
        let temp_max = self.temperature_max;
        let temp: u8 = biome.temperature.into();

        if temp < temp_max {
            let humid_max = self.humidity_max;
            let humid: u8 = biome.humidity.into();

            if humid < humid_max {
                let temp_u = temp as usize;
                let humid_u = humid as usize;
                let humid_max_u = humid_max as usize;
                let index = temp_u * humid_max_u + humid_u;

                self.biomes[index] = biome;
            }
        }
    }
}
