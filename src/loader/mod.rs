use std::fs::File;
use bevy::prelude::*;

pub mod official;
pub mod phichain;

pub trait Loader {
    fn load(file: File, commands: &mut Commands);
}
