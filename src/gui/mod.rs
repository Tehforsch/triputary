mod config;
mod cutting_thread;
mod plot;
mod playback;

use crate::{cut::CutInfo, excerpt_collection::ExcerptCollection, excerpt_collections::ExcerptCollections, song::Song};
use eframe::{egui::{self, Align, Color32, Label, Layout, Ui}, epi};

use self::{cutting_thread::CuttingThreadHandle, playback::{PlaybackThreadHandle, play_excerpt}, plot::ExcerptPlot};

pub struct StriputaryGui {
    collections: ExcerptCollections,
    plots: Vec<ExcerptPlot>,
    cut_thread: CuttingThreadHandle,
    current_playback: Option<PlaybackThreadHandle>,
}

impl StriputaryGui {
    pub fn new(collections: ExcerptCollections) -> Self {
        let collection = collections.get_selected();
        let plots = StriputaryGui::get_plots(collection);
        let thread = CuttingThreadHandle::default();
        let current_playback = None;
        Self {
            collections,
            plots,
            cut_thread: thread,
            current_playback,
        }
    }

    fn cut_songs(&self) {
        let cut_infos = self.get_cut_info();
        self.cut_thread.send_cut_infos(cut_infos);
    }

    fn get_cut_info(&self) -> Vec<CutInfo> {
        let collection = self.collections.get_selected();
        let mut cut_info: Vec<CutInfo> = vec![];
        for (plot_start, plot_end) in self.plots.iter().zip(self.plots[1..].iter()) {
            let song = plot_start.excerpt.song_after.as_ref().unwrap();
            cut_info.push(CutInfo::new(
                &collection.session,
                song.clone(),
                plot_start.cut_time,
                plot_end.cut_time,
            ));
        }
        cut_info
    }

    fn mark_cut_songs(&mut self) {
        let cut_songs = self.cut_thread.get_cut_songs();
        for song in cut_songs {
            for plot in self.plots.iter_mut() {
                if let Some(ref song_before) = plot.excerpt.song_before {
                    if song_before == song {
                        plot.finished_cutting_song_before = true;
                    }
                }
                if let Some(ref song_after) = plot.excerpt.song_after {
                    if song_after == song {
                        plot.finished_cutting_song_after = true;
                    }
                }
            }
        }
    }

    fn select(&mut self, selection: usize) {
        self.collections.select(selection);
        self.plots = StriputaryGui::get_plots(self.collections.get_selected());
    }

    fn get_plots(collection: &ExcerptCollection) -> Vec<ExcerptPlot> {
        collection
            .iter_excerpts()
            .map(|excerpt| {
                ExcerptPlot::new(
                    excerpt.clone(),
                    excerpt
                        .excerpt
                        .get_absolute_time_from_time_offset(collection.offset_guess),
                )
            })
            .collect()
    }

    pub fn get_label_color(finished_cutting: bool) -> Color32 {
        match finished_cutting {
            true => config::CUT_LABEL_COLOR,
            false => config::UNCUT_LABEL_COLOR,
        }
    }


    fn add_plot_label(ui: &mut Ui, song: Option<&Song>, finished_cutting: bool) {
        let color = StriputaryGui::get_label_color(finished_cutting);
        if let Some(ref song) = song {
            ui.add(Label::new(format!("{}", song.title)).text_color(color));
        }
    }

    fn add_top_bar(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                let mut selection: Option<usize> = None;
                for (i, collection) in self.collections.enumerate() {
                    if ui.button(collection.name()).clicked() {
                        selection = Some(i);
                    }
                }
                if let Some(selection) = selection {
                    self.select(selection);
                }
            });
        });
    }

    fn add_side_bar(&mut self, ctx: &egui::CtxRef) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if ui.button("Cut").clicked() {
                self.cut_songs();
            }
            if ui.button("Playback").clicked() {
                let plot = &self.plots[0];
                let excerpt = &plot.excerpt.excerpt;
                if let Some(ref thread) = self.current_playback {
                    thread.shut_down();
                }
                self.current_playback = Some(play_excerpt(excerpt, excerpt.get_relative_time(plot.cut_time)));
            }
        });
    }

    fn add_central_panel(&mut self, ctx: &egui::CtxRef) {
        egui::CentralPanel::default().show(ctx, |ui| {
            for plot in self.plots.iter_mut() {
                ui.horizontal(|ui| {
                    Self::add_plot_label(ui, plot.excerpt.song_before.as_ref(), plot.finished_cutting_song_before);
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        Self::add_plot_label(ui, plot.excerpt.song_after.as_ref(), plot.finished_cutting_song_after);
                    });
                });
                ui.add(plot);
            }
            egui::warn_if_debug_build(ui);
        });
    }
}

impl epi::App for StriputaryGui {
    fn name(&self) -> &str {
        "Striputary"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        self.add_top_bar(ctx);
        self.add_side_bar(ctx);

        self.add_central_panel(ctx);

        self.mark_cut_songs();
    }
}
