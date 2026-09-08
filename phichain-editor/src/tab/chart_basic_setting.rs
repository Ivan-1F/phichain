use crate::editing::history::Edits;
use bevy::prelude::*;
use egui::Ui;
use phichain_chart::id::ProjectId;
use phichain_chart::offset::Offset;
use phichain_chart::project::ProjectMeta;

pub fn chart_basic_setting_tab(
    In(mut ui): In<Ui>,
    settings: Single<(Entity, &Offset, &ProjectMeta), With<ProjectId>>,
    mut edits: Edits,
) {
    let (entity, offset, meta) = settings.into_inner();
    edits
        .component(entity, offset, t!("history.edit_offset"))
        .field(
            &mut ui,
            "offset",
            t!("tab.chart_basic_setting.offset"),
            |ui, offset| {
                ui.add(egui::DragValue::new(&mut offset.0).speed(1));
            },
        );
    let mut editor = edits.component(entity, meta, t!("history.edit_meta"));
    editor.field(
        &mut ui,
        "name",
        t!("tab.chart_basic_setting.name"),
        |ui, meta| {
            ui.add(egui::TextEdit::singleline(&mut meta.name));
        },
    );
    editor.field(
        &mut ui,
        "level",
        t!("tab.chart_basic_setting.level"),
        |ui, meta| {
            ui.add(egui::TextEdit::singleline(&mut meta.level));
        },
    );
    editor.field(
        &mut ui,
        "composer",
        t!("tab.chart_basic_setting.composer"),
        |ui, meta| {
            ui.add(egui::TextEdit::singleline(&mut meta.composer));
        },
    );
    editor.field(
        &mut ui,
        "charter",
        t!("tab.chart_basic_setting.charter"),
        |ui, meta| {
            ui.add(egui::TextEdit::singleline(&mut meta.charter));
        },
    );
    editor.field(
        &mut ui,
        "illustrator",
        t!("tab.chart_basic_setting.illustrator"),
        |ui, meta| {
            ui.add(egui::TextEdit::singleline(&mut meta.illustrator));
        },
    );
}
