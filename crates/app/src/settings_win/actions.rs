use super::*;

pub(super) unsafe fn dispatch(hwnd: HWND, act: Option<Action>) {
  match act {
    Some(Action::EditText(f)) => {
      with_win(|w| {
        if w.edit != Some(f) {
          if w.edit.is_some() {
            w.commit_edit();
          }
          let s = prefs::get();
          if (8..=11).contains(&f) {
            let kind = match f {
              8 => ListKind::ExcludedPrograms,
              9 => ListKind::ProgramBlacklist,
              10 => ListKind::MinimizeBlacklist,
              11 => ListKind::DeletedStartMenuPrograms,
              _ => return,
            };
            let lists = prefs::lists();
            w.edit_buf = lists.to_text(kind);
          } else {
            w.edit_buf = if f == 0 {
              s.greet_name.as_str().to_string()
            } else {
              s.greet_text.as_str().to_string()
            };
          }
          w.edit = Some(f);
          w.render();
        }
      });
      let _ = SetForegroundWindow(hwnd);
      let _ = SetFocus(Some(hwnd));
    }
    Some(Action::NotchIdle(v)) => prefs::update(|s| s.notch_idle = v),
    Some(Action::LyricsCompact) => prefs::update(|s| s.lyrics_compact = !s.lyrics_compact),
    Some(Action::NotchDownloads) => prefs::update(|s| s.notch_downloads = !s.notch_downloads),
    Some(Action::NotchSend) => prefs::update(|s| s.notch_send = !s.notch_send),
    Some(Action::NotchMedia) => prefs::update(|s| s.extra.notch_media = !s.extra.notch_media),
    Some(Action::NotchShelf) => prefs::update(|s| s.extra.notch_shelf = !s.extra.notch_shelf),
    Some(Action::NotchTimer) => prefs::update(|s| s.extra.notch_timer = !s.extra.notch_timer),
    Some(Action::LyricsOnline) => prefs::update(|s| s.lyrics_online = !s.lyrics_online),
    Some(Action::Donate) => nexora_platform::launch::open(DONATE_URL),
    Some(Action::MenuLogoPick) => {
      let _ = SetWindowPos(
        hwnd,
        Some(HWND_NOTOPMOST),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      );
      let picked = nexora_platform::pick::open_file(
        prefs::tr("Pilih gambar"),
        &[(
          prefs::tr("Gambar logo"),
          "*.png;*.jpg;*.jpeg;*.bmp;*.ico;*.gif;*.webp;*.tif;*.tiff",
        )],
      );
      let _ = SetWindowPos(
        hwnd,
        Some(HWND_TOPMOST),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      );
      if let Some(p) = picked {
        if crate::menubar::logo::set(&p) {
          prefs::broadcast();
        }
      }
    }
    Some(Action::UiScale(v)) => prefs::update(|s| s.ui_scale = v),
    Some(Action::WidgetStyle(v)) => prefs::update(|s| s.widget_style = v),
    Some(Action::BatteryPct(v)) => prefs::update(|s| s.battery_pct = v),
    Some(Action::WidgetGallery) => crate::widgets::open_gallery(),
    Some(Action::NotchShape(b)) => prefs::update(|s| s.notch_island = b),
    // M2 DLB handlers
    Some(Action::DockIconEffect(v)) => prefs::update(|s| s.extra.dock_icon_effect = v),
    Some(Action::DockPosition(v)) => prefs::update(|s| s.extra.dock_position = v),
    Some(Action::DockDisplay(v)) => prefs::update(|s| s.extra.dock_display = v as u16),
    // M2 DLP handlers
    Some(Action::LaunchpadSources(v)) => prefs::update(|s| s.extra.launchpad_sources = v),
    Some(Action::LaunchpadDisplay(v)) => prefs::update(|s| s.extra.launchpad_display = v as u16),
    Some(Action::LaunchpadFollowDock) => {
      prefs::update(|s| s.extra.launchpad_follow_dock = !s.extra.launchpad_follow_dock)
    }
    // M2 DAD handlers
    Some(Action::DockDblClickFolder) => prefs::update(|s| s.extra.dock_dblclick_folder = !s.extra.dock_dblclick_folder),
    Some(Action::DockHideOpenFolder) => {
      prefs::update(|s| s.extra.dock_hide_open_folder_button = !s.extra.dock_hide_open_folder_button)
    }
    Some(Action::DockLockDrag) => prefs::update(|s| s.extra.dock_lock_dragging = !s.extra.dock_lock_dragging),
    Some(Action::DockShowLabels) => prefs::update(|s| s.extra.dock_show_labels = !s.extra.dock_show_labels),
    Some(Action::DockAnimateLaunch) => prefs::update(|s| s.extra.dock_animate_launch = !s.extra.dock_animate_launch),
    Some(Action::DockPreventCover) => prefs::update(|s| s.extra.dock_prevent_cover = !s.extra.dock_prevent_cover),
    Some(Action::DockShowIndicators) => prefs::update(|s| s.extra.dock_show_indicators = !s.extra.dock_show_indicators),
    Some(Action::DockShowUnpinned) => prefs::update(|s| s.extra.dock_show_unpinned = !s.extra.dock_show_unpinned),
    Some(Action::DockSeparateUnpinned) => {
      prefs::update(|s| s.extra.dock_separate_unpinned = !s.extra.dock_separate_unpinned)
    }
    Some(Action::DockDetection(v)) => prefs::update(|s| s.extra.dock_detection = v),
    Some(Action::DockHiddenEffect(v)) => prefs::update(|s| s.extra.dock_hidden_effect = v),
    Some(Action::DockActivationTrigger(v)) => prefs::update(|s| s.extra.dock_activation_trigger = v),
    // M2 DMN handlers
    Some(Action::UseNexoraMinimize) => prefs::update(|s| s.extra.use_nexora_minimize = !s.extra.use_nexora_minimize),
    Some(Action::DisableMinimizeAnim) => {
      prefs::update(|s| s.extra.disable_minimize_anim = !s.extra.disable_minimize_anim)
    }
    Some(Action::ShiftSlowAnim) => prefs::update(|s| s.extra.shift_slow_anim = !s.extra.shift_slow_anim),
    Some(Action::MinimizeIntoIcon) => prefs::update(|s| s.extra.minimize_into_icon = !s.extra.minimize_into_icon),
    Some(Action::DynamicMinimizeContent) => {
      prefs::update(|s| s.extra.dynamic_minimize_content = !s.extra.dynamic_minimize_content)
    }
    Some(Action::MinimizedIconPos(v)) => prefs::update(|s| s.extra.minimized_icon_position = v),
    Some(Action::MinimizeAnimSpeed(v)) => prefs::update(|s| s.extra.minimize_anim_speed = v),
    // M2 DPV handlers
    Some(Action::WindowPreviews) => prefs::update(|s| s.extra.window_previews = !s.extra.window_previews),
    Some(Action::PreviewAppName) => prefs::update(|s| s.extra.preview_app_name = !s.extra.preview_app_name),
    Some(Action::PreviewControls) => prefs::update(|s| s.extra.preview_controls = !s.extra.preview_controls),
    Some(Action::PreviewNative) => prefs::update(|s| s.extra.preview_native = !s.extra.preview_native),
    // M2 STG handlers
    Some(Action::StageIcon) => prefs::update(|s| s.extra.stage_icon = !s.extra.stage_icon),
    Some(Action::StageBlurIcons) => prefs::update(|s| s.extra.stage_blur_icons = !s.extra.stage_blur_icons),
    Some(Action::StageNativePreview) => prefs::update(|s| s.extra.stage_native_preview = !s.extra.stage_native_preview),
    Some(Action::StageKeepBorder) => prefs::update(|s| s.extra.stage_keep_border = !s.extra.stage_keep_border),
    Some(Action::StageHiddenMode(v)) => prefs::update(|s| s.extra.stage_hidden_mode = v),
    // M2 FLB handlers
    Some(Action::FinderMenuIcon) => prefs::update(|s| s.extra.finder_menu_icon = !s.extra.finder_menu_icon),
    Some(Action::FinderAutoHide) => prefs::update(|s| s.extra.finder_auto_hide = !s.extra.finder_auto_hide),
    Some(Action::FinderShutdownConfirm) => {
      prefs::update(|s| s.extra.finder_shutdown_confirmation = !s.extra.finder_shutdown_confirmation)
    }
    Some(Action::FinderMaximizedAffects) => {
      prefs::update(|s| s.extra.finder_maximized_affects_appearance = !s.extra.finder_maximized_affects_appearance)
    }
    Some(Action::FinderCloseMaximizedCorner) => {
      prefs::update(|s| s.extra.finder_close_maximized_on_corner = !s.extra.finder_close_maximized_on_corner)
    }
    Some(Action::FinderLockTrayDrag) => {
      prefs::update(|s| s.extra.finder_lock_tray_dragging = !s.extra.finder_lock_tray_dragging)
    }
    Some(Action::FinderWindowOptions) => {
      prefs::update(|s| s.extra.finder_window_options = !s.extra.finder_window_options)
    }
    Some(Action::FinderColoredTray) => {
      prefs::update(|s| s.extra.finder_colored_tray_icons = !s.extra.finder_colored_tray_icons)
    }
    Some(Action::FinderInputMethod) => {
      prefs::update(|s| s.extra.finder_input_method_menu = !s.extra.finder_input_method_menu)
    }
    Some(Action::FinderSearchIcon) => prefs::update(|s| s.extra.finder_search_icon = !s.extra.finder_search_icon),
    Some(Action::FinderAccountIcon) => prefs::update(|s| s.extra.finder_account_icon = !s.extra.finder_account_icon),
    Some(Action::FinderMediaPlayback) => {
      prefs::update(|s| s.extra.finder_media_playback_icon = !s.extra.finder_media_playback_icon)
    }
    Some(Action::FinderLyricsMenu) => prefs::update(|s| s.extra.finder_lyrics_menu = !s.extra.finder_lyrics_menu),
    Some(Action::FinderColorLyrics) => {
      prefs::update(|s| s.extra.finder_color_player_lyrics = !s.extra.finder_color_player_lyrics)
    }
    // M2 FNO handlers
    Some(Action::FinderShowNotif) => {
      prefs::update(|s| s.extra.finder_show_notifications = !s.extra.finder_show_notifications)
    }
    Some(Action::FinderNotifSounds) => {
      prefs::update(|s| s.extra.finder_notification_sounds = !s.extra.finder_notification_sounds)
    }
    // M2 FTD handlers
    Some(Action::FinderShowTime) => prefs::update(|s| s.extra.finder_show_time = !s.extra.finder_show_time),
    Some(Action::FinderShowDate) => prefs::update(|s| s.extra.finder_show_date = !s.extra.finder_show_date),
    Some(Action::FinderDateFormat(v)) => prefs::update(|s| s.extra.finder_date_format = v),
    // M2 FAU handlers
    Some(Action::FinderShowVolume) => prefs::update(|s| s.extra.finder_show_volume = !s.extra.finder_show_volume),
    Some(Action::FinderShowAudioOutput) => {
      prefs::update(|s| s.extra.finder_show_audio_output = !s.extra.finder_show_audio_output)
    }
    Some(Action::FinderShowMediaControls) => {
      prefs::update(|s| s.extra.finder_show_media_controls = !s.extra.finder_show_media_controls)
    }
    // M2 FDI handlers
    Some(Action::FinderShowBrightness) => {
      prefs::update(|s| s.extra.finder_show_brightness = !s.extra.finder_show_brightness)
    }
    // M2 FHW handlers
    Some(Action::FinderShowWifi) => prefs::update(|s| s.extra.finder_show_wifi = !s.extra.finder_show_wifi),
    Some(Action::FinderShowBluetooth) => {
      prefs::update(|s| s.extra.finder_show_bluetooth = !s.extra.finder_show_bluetooth)
    }
    Some(Action::FinderShowHardware) => {
      prefs::update(|s| s.extra.finder_show_hardware_icons = !s.extra.finder_show_hardware_icons)
    }
    // M2 FBA handlers
    Some(Action::FinderShowBattery) => {
      prefs::update(|s| s.extra.finder_show_battery_menu = !s.extra.finder_show_battery_menu)
    }
    Some(Action::FinderRemainingBattery) => {
      prefs::update(|s| s.extra.finder_remaining_battery_time = !s.extra.finder_remaining_battery_time)
    }
    Some(Action::FinderAutoPowerModes) => {
      prefs::update(|s| s.extra.finder_auto_power_modes = !s.extra.finder_auto_power_modes)
    }
    Some(Action::FinderPowerPlugged(v)) => prefs::update(|s| s.extra.finder_power_mode_plugged = v),
    Some(Action::FinderPowerBattery(v)) => prefs::update(|s| s.extra.finder_power_mode_battery = v),
    // M2 APP handlers
    Some(Action::CustomDockBlur) => prefs::update(|s| s.extra.custom_dock_blur = !s.extra.custom_dock_blur),
    Some(Action::PinnedReflections) => prefs::update(|s| s.extra.pinned_reflections = !s.extra.pinned_reflections),
    Some(Action::CustomIconLabels) => prefs::update(|s| s.extra.custom_icon_labels = !s.extra.custom_icon_labels),
    Some(Action::CustomContextMenus) => prefs::update(|s| s.extra.custom_context_menus = !s.extra.custom_context_menus),
    Some(Action::CustomFolderGrid) => prefs::update(|s| s.extra.custom_folder_grid = !s.extra.custom_folder_grid),
    Some(Action::CustomNexoraMenus) => prefs::update(|s| s.extra.custom_nexora_menus = !s.extra.custom_nexora_menus),
    Some(Action::CustomFinderBlur) => prefs::update(|s| s.extra.custom_finder_blur = !s.extra.custom_finder_blur),
    Some(Action::LaunchpadCustomBlur) => {
      prefs::update(|s| s.extra.launchpad_custom_blur = !s.extra.launchpad_custom_blur)
    }
    Some(Action::LaunchpadAboveWindows) => {
      prefs::update(|s| s.extra.launchpad_above_windows = !s.extra.launchpad_above_windows)
    }
    Some(Action::LaunchpadBlurBackground) => {
      prefs::update(|s| s.extra.launchpad_blur_background = !s.extra.launchpad_blur_background)
    }
    Some(Action::TrayAllFinder) => prefs::update(|s| s.extra.tray_all_finder = !s.extra.tray_all_finder),
    Some(Action::TrayAllDock) => prefs::update(|s| s.extra.tray_all_dock = !s.extra.tray_all_dock),
    Some(Action::TraySystemFinder) => prefs::update(|s| s.extra.tray_system_finder = !s.extra.tray_system_finder),
    Some(Action::NightLight) => prefs::update(|s| s.extra.night_light = !s.extra.night_light),
    // M2 SCR handlers
    Some(Action::ScreenTarget(v)) => prefs::update(|s| s.extra.screen_target = v),
    // M2 GEN handlers
    Some(Action::StartMethod(v)) => prefs::update(|s| s.extra.start_method = v),
    Some(Action::TaskbarScope(v)) => prefs::update(|s| s.extra.taskbar_scope = v),
    Some(Action::DisplayFont(v)) => prefs::update(|s| s.extra.display_font = v),
    // M2 WAN handlers
    Some(Action::WindowAnimEffect(v)) => prefs::update(|s| s.extra.window_anim_effect = v),
    Some(Action::WindowAnimSpeed(v)) => prefs::update(|s| s.extra.window_anim_speed = v),
    Some(Action::MinimizeExcludedWindows) => {
      prefs::update(|s| s.extra.minimize_excluded_windows = !s.extra.minimize_excluded_windows)
    }
    Some(Action::MenuLogoReset) => {
      crate::menubar::logo::reset();
      prefs::broadcast();
    }
    Some(Action::LyricsDir) => {
      let dir = crate::paths::config_dir().join("lyrics");
      let _ = std::fs::create_dir_all(&dir);
      nexora_platform::launch::open(&dir.to_string_lossy());
    }
    Some(Action::Search) => {
      with_win(|w| {
        w.search_focus = true;
        w.render();
      });
      let _ = SetForegroundWindow(hwnd);
      let _ = SetFocus(Some(hwnd));
    }
    Some(Action::Back) => {
      with_win(|w| {
        if w.hpos > 0 {
          w.hpos -= 1;
          let t = w.history[w.hpos];
          w.go(t, false);
        }
      });
    }
    Some(Action::Forward) => {
      with_win(|w| {
        if w.hpos + 1 < w.history.len() {
          w.hpos += 1;
          let t = w.history[w.hpos];
          w.go(t, false);
        }
      });
    }
    Some(Action::IconVariant(v)) => prefs::update(|s| s.icon_variant = v),
    Some(Action::MinEffect(v)) => prefs::update(|s| s.min_effect = v),
    Some(Action::Close) => {
      let _ = PostMessageW(Some(hwnd), WM_CLOSE_SELF, WPARAM(0), LPARAM(0));
    }
    Some(Action::Takeover) => prefs::update(|s| s.takeover = !s.takeover),
    Some(Action::Dock(b)) => prefs::update(|s| s.dock = b),
    Some(Action::Notch(b)) => prefs::update(|s| s.notch = b),
    Some(Action::Theme(t)) => prefs::update(|s| s.theme = t),
    Some(Action::Lang(l)) => prefs::update(|s| s.lang = l),
    Some(Action::WinShadow) => prefs::update(|s| s.win_shadow = !s.win_shadow),
    Some(Action::HoverOpen) => prefs::update(|s| s.notch_hover_open = !s.notch_hover_open),
    Some(Action::MenuBarOn) => prefs::update(|s| s.menubar = !s.menubar),
    Some(Action::HotCycle(i)) => prefs::update(|s| s.hot[i] = s.hot[i].next()),
    Some(Action::StageOn) => prefs::update(|s| s.stage = !s.stage),
    Some(Action::SwitcherOn) => prefs::update(|s| s.switcher = !s.switcher),
    Some(Action::StageFlag(k)) => prefs::update(|s| match k {
      1 => s.stage_desktop_click = !s.stage_desktop_click,
      2 => s.stage_media = !s.stage_media,
      _ => s.stage_titles = !s.stage_titles,
    }),
    Some(Action::StageRight(r)) => prefs::update(|s| s.stage_right = r),
    Some(Action::StageIcons(m)) => prefs::update(|s| s.stage_icons = m),
    Some(Action::MacKeys) => prefs::update(|s| s.mac_shortcuts = !s.mac_shortcuts),
    Some(Action::Accent(i)) => prefs::update(|s| s.accent = i),
    Some(Action::IconStyle(m)) => prefs::update(|s| {
      s.icon_style = match m {
        0 => nexora_core::settings::IconStyle::Default,
        1 => nexora_core::settings::IconStyle::Dark,
        2 => nexora_core::settings::IconStyle::Clear,
        _ => nexora_core::settings::IconStyle::Tinted,
      }
    }),
    Some(Action::IconTint(i)) => prefs::update(|s| s.icon_tint = i),
    Some(Action::ClipHistory) => prefs::update(|s| s.clipboard_history = !s.clipboard_history),
    Some(Action::AboveFull) => prefs::update(|s| s.above_fullscreen = !s.above_fullscreen),
    Some(Action::DockPlates) => prefs::update(|s| s.dock_plates = !s.dock_plates),
    Some(Action::IconPack) => prefs::update(|s| s.icon_pack = !s.icon_pack),
    Some(Action::DownloadsStack) => prefs::update(|s| s.downloads_stack = !s.downloads_stack),
    Some(Action::RecentApps) => prefs::update(|s| s.recent_apps = !s.recent_apps),
    Some(Action::HideIcons) => prefs::update(|s| s.hide_desktop_icons = !s.hide_desktop_icons),
    Some(Action::BarStyle(m)) => prefs::update(|s| match m {
      0 => {
        s.menubar_glass = false;
        s.menubar_material = false;
      }
      3 => {
        s.menubar_glass = false;
        s.menubar_material = true;
      }
      1 => {
        s.menubar_material = false;
        s.menubar_glass = true;
        s.menubar_style = nexora_core::settings::BarStyle::Light;
      }
      _ => {
        s.menubar_material = false;
        s.menubar_glass = true;
        s.menubar_style = nexora_core::settings::BarStyle::Dark;
      }
    }),
    Some(Action::Clock24) => prefs::update(|s| s.clock_24h = !s.clock_24h),
    Some(Action::DockOn) => prefs::update(|s| s.dock_enabled = !s.dock_enabled),
    Some(Action::NotchOn) => prefs::update(|s| s.notch_enabled = !s.notch_enabled),
    Some(Action::Refraction) => prefs::update(|s| s.refraction = !s.refraction),
    Some(Action::Anim) => prefs::update(|s| s.anim = !s.anim),
    Some(Action::Autostart) => prefs::update(|s| s.autostart = !s.autostart),
    Some(Action::Material(true, m)) => prefs::update(|s| s.dock_material = m),
    Some(Action::Material(false, m)) => prefs::update(|s| s.notch_material = m),
    Some(Action::Tab(t)) => {
      with_win(|w| w.go(t, true));
    }
    _ => {}
  }
}
