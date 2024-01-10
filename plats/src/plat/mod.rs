mod deserialize;
mod layer;
mod level;
mod node;
mod serialize;
mod smc;

use std::{
    fs::{create_dir_all, File},
    io::Write,
};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use self::{layer::Layer, smc::SmallBlockCollection};

#[derive(Default, Serialize, Deserialize)]
pub struct Plat {
    pub position: (i32, i32, i32),
    pub rotation: (i32, i32, i32),
    pub bcs: Vec<()>,
    pub texs: Vec<()>,
    pub sbc: SmallBlockCollection,
    pub depth: u8,
    pub layer_limit: u8,
    #[serde(skip)]
    pub layers: Vec<Layer>,
}

impl Plat {
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        create_dir_all(format!("{}.plat", path))?;
        let entry: String = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default())?;
        let mut file = File::create(format!("{}.plat/entry.ron", path))?;
        file.write_all(&entry.as_bytes())?;
        // Create layers dirs
        for layer in &self.layers {
            let layer_path = format!("{}.plat/layers/{}.layer", path, layer.name);
            create_dir_all(&layer_path)?;
            for (level_id, level) in layer.levels.iter().enumerate() {
                // let level_stringified: String =
                //     ron::ser::to_string_pretty(&level, ron::ser::PrettyConfig::default())?;
                let encoded: Vec<u8> = bitcode::encode(&level).unwrap();

                let mut file = File::create(format!("{}/{}-level", layer_path, level_id))?;
                file.write_all(&encoded)?;
            }
        }
        Ok(())
    }

    pub fn load(path: &str) -> anyhow::Result<Self> {
        todo!()
    }

    pub fn from_width(min_width: u32) -> anyhow::Result<Self> {
        todo!()
    }

    pub fn new(depth: u8) -> anyhow::Result<Self> {
        if depth > 25 {
            bail!("You cant create plat with depth {}. You will never ever need world with size 2^{} ({})", depth, depth, 1 << depth)
        }
        Ok(Self {
            layer_limit: 5,
            depth,
            ..Default::default()
        })
    }
}

#[test]
fn test_serialization() {
    let mut pl = Plat::new(5).unwrap();
    pl.add_layer("base").unwrap();
    pl.add_layer("artificial").unwrap();
    //dbg!(&pl.layers[0]);
    pl.save("./saves/my_first_world").unwrap();
}

#[test]
fn create_new() {
    assert!(Plat::new(26).is_err());
}