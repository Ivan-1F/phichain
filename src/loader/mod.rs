use std::fs::File;
use bevy::prelude::*;

pub mod official;

pub trait Loader {
    fn load(file: File, commands: Commands);
}
