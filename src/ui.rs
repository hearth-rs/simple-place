// use eigen_or_whatever::{Quat, V3, V2i as Coord};
use wgpu::Texture;
use crate::{App};
use std::rc::{Rc};
use egui::{Modifiers, Key, Event};
use crate::conf::Conf;


struct Tileset {
    texture: Texture,
    tile_span: usize,
    dimensions: (usize, usize),
    tiles_per_row: usize,
    total_tile_n: usize,
}

type TileIndex = usize;

#[derive(Clone)]
pub struct Material {
    tileset: Rc<Tileset>,
    preview_tile: TileIndex,
}

#[derive(Clone)]
pub enum EditorMode {
    Material{
        selected: Option<Rc<Material>>,
    },
    Tiles{
        
    }
}

pub struct UserInterface {
    pub mode: EditorMode,
    pub conf: Conf,
    //for claiming access to the mouse in a way that's somewhat debugable if someone doesn't clean up
    pub mouse_lockout: Vec<&'static str>,
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Material{selected:None},
            conf: Conf::default(),
            mouse_lockout: Vec::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello, world!");
        });
    }
}
