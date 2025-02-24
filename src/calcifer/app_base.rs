use std::{path::PathBuf, fs, path::Path, cmp::min, io};
use eframe::egui;
use egui::Color32;

use crate::Calcifer;
use crate::tools;
use crate::PATH_ROOT;
use crate::DEFAULT_THEMES;
use crate::MAX_TABS;
use crate::SAVE_PATH;
use crate::TIME_LABELS;


impl Calcifer {
	pub fn handle_confirm(&mut self) {
		if self.close_tab_confirm.proceed {
			self.close_tab_confirm.close();
			self.delete_tab(self.tab_to_close);
		}
		
		if self.refresh_confirm.proceed {
			self.refresh_confirm.close();
			self.tabs[self.selected_tab.to_index()].refresh();
		}
	}
	
	pub fn save_tab(&self) -> Option<PathBuf> {
		if self.tabs[self.selected_tab.to_index()].path.file_name().expect("Could not get Tab Name").to_string_lossy().to_string() == "untitled" {
			return self.save_tab_as();
		} else {
			if let Err(err) = fs::write(&self.tabs[self.selected_tab.to_index()].path, &self.tabs[self.selected_tab.to_index()].code) {
				eprintln!("Error writing file: {}", err);
				return None;
			}
			return Some(self.tabs[self.selected_tab.to_index()].path.clone())
		}
	}
	
	
	pub fn save_tab_as(&self) -> Option<PathBuf> {
		if let Some(path) = rfd::FileDialog::new().set_directory(Path::new(&PATH_ROOT)).save_file() {
			if let Err(err) = fs::write(&path, &self.tabs[self.selected_tab.to_index()].code) {
				eprintln!("Error writing file: {}", err);
				return None;
			}
			return Some(path);
		}
		return None
	}
	
	
	pub fn handle_save_file(&mut self, path_option : Option<PathBuf>) {
		if let Some(path) = path_option {
			println!("File saved successfully at: {:?}", path);
			self.tabs[self.selected_tab.to_index()].path = path;
			self.tabs[self.selected_tab.to_index()].saved = true;
		} else {
			println!("File save failed.");
		}
	}
	
	
	pub fn from_app_state(app_state: tools::AppState) -> Self {
		let mut new = Self {
			theme: DEFAULT_THEMES[min(app_state.theme, DEFAULT_THEMES.len() - 1)],
			tabs: Vec::new(),
			settings_menu: tools::settings::SettingsWindow::new(DEFAULT_THEMES[app_state.theme]),
			..Default::default()
		};
		
		for path in app_state.tabs {
			if path.file_name().expect("Could not get Tab Name").to_string_lossy().to_string() != "untitled" {
				new.open_file(Some(&path));
			}
		}
		
		if new.tabs == vec![] {
			new.open_file(None);
		}
		
		new
	}
	
	
	pub fn save_state(&self) {
		let mut state_theme : usize = 0;
		if let Some(theme) = DEFAULT_THEMES.iter().position(|&r| r == self.theme) {
			state_theme = theme;
		}
		
		let mut state_tabs = vec![];
		
		for tab in &self.tabs {
			state_tabs.push(tab.path.clone());
		}
		let app_state = tools::AppState {
			tabs: state_tabs,
			theme: state_theme,
		};
		
		let _ = tools::save_state(&app_state, SAVE_PATH);
	}
	
	
	pub fn move_through_tabs(&mut self, forward : bool) {
		let new_index = if forward {
			(self.selected_tab.to_index() + 1) % self.tabs.len()
		} else {
			self.selected_tab.to_index().checked_sub(1).unwrap_or(self.tabs.len() - 1)
		};
		self.selected_tab = tools::TabNumber::from_index(new_index);
	}
	
	
	pub fn list_files(&mut self, ui: &mut egui::Ui, path: &Path) -> io::Result<()> {
		if let Some(name) = path.file_name() {
			if path.is_dir() {
				egui::CollapsingHeader::new(name.to_string_lossy()).show(ui, |ui| {
					let mut paths: Vec<_> = fs::read_dir(&path).expect("Failed to read dir").map(|r| r.unwrap()).collect();
					
					paths.sort_by(|a, b| tools::sort_directories_first(a, b));

					for result in paths {
						let _ = self.list_files(ui, &result.path());
					}
				});
			} else {
				if ui.button(name.to_string_lossy()).clicked() {
					self.open_file(Some(path));
				}
			}
		}
		Ok(())
	}
	
	
	pub fn open_file(&mut self, path_option: Option<&Path>) {
		if self.tabs.len() < MAX_TABS {
			if let Some(path) = path_option {
				self.tabs.push(tools::Tab::new(path.to_path_buf()));
			} else {
				self.tabs.push(tools::Tab::default());
			}
			self.selected_tab = tools::TabNumber::from_index(self.tabs.len() - 1);
		}
	}
	
	
	pub fn delete_tab(&mut self, index : usize) {
		self.tabs.remove(index);
		self.selected_tab = tools::TabNumber::from_index(min(index, self.tabs.len() - 1));
	}
	
	
	pub fn toggle(&self, ui: &mut egui::Ui, display : bool, title : &str) -> bool {
		let bg_color : Color32;
		let text_color : Color32;
		
		if display.clone() {
			bg_color = Color32::from_hex(self.theme.functions).expect("Could not convert color to hex (functions)");
			text_color = Color32::from_hex(self.theme.bg).expect("Could not convert color to hex (bg)");
		} else {
			bg_color = Color32::from_hex(self.theme.bg).expect("Could not convert color to hex (bg)");
			text_color = Color32::from_hex(self.theme.literals).expect("Could not convert color to hex (literals)");
		};
		
		ui.style_mut().visuals.override_text_color = Some(text_color);
		
		if ui.add(egui::Button::new(title).fill(bg_color)).clicked() {
			return !display
		}
		ui.style_mut().visuals.override_text_color = None;
		
		return display
	}
	
	pub fn profiler(&self) -> String {
		if !self.profiler_visible {
			return "".to_string()
		}
		let combined_string: Vec<String> = TIME_LABELS.into_iter().zip(self.time_watch.clone().into_iter())
			.map(|(s, v)| format!("{} : {:.1} ms", s, v)).collect();

		let mut result = combined_string.join(" ;  ");
		result.push_str(&format!("	total : {:.1} ms", self.time_watch.clone().iter().sum::<f32>()));
		return result
	}
}