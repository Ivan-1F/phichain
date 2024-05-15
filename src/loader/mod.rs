use bevy::prelude::*;
use std::fs::File;

pub mod official;
pub mod phichain;

pub trait Loader {
    fn load(file: File, commands: &mut Commands);
}
