#include "main_frame.hpp"

#include "enums.hpp"
#include "paths.hpp"
#include "strings.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/bootstrap_panel.hpp"
#include "ui/panels/connection_status_panel.hpp"
#include "ui/panels/file_transfers_panel.hpp"
#include "ui/panels/new_profile_panel.hpp"
#include "ui/panels/sessions_notebook.hpp"
#include "ui/panels/settings_panel.hpp"
#include "ui/widgets/wrapped_static_text.hpp"

MainFrame::MainFrame() : wxFrame(nullptr, wxID_ANY, Strings::MainFrame::title()) {}

void MainFrame::init() {
    this->setup_menubar();

    auto sizer = new wxBoxSizer(wxHORIZONTAL);
    this->setup_main_panels(sizer);
    this->setup_overlay_panels(sizer);

    this->SetSizerAndFit(sizer);
    this->SetMinSize(wxSize(800, 600));
    this->SetSize(wxSize(800, 600));

    // Bind Event Handlers
    this->Bind(wxEVT_MENU, [this](wxCommandEvent&) { this->Close(true); }, wxID_EXIT);

    this->show_bootstrap_panel();
}

//
// Show/Hide panel functions
//

void MainFrame::show_bootstrap_panel() {
    this->show_main_panel(this->main_panels.bootstrap_panel);
}

void MainFrame::show_sessions_notebook_panel() {
    this->show_main_panel(this->main_panels.sessions_notebook_panel);
}

void MainFrame::show_file_transfers_panel() {
    this->show_overlay_panel(this->overlay_panels.file_transfers_panel);
}

void MainFrame::show_settings_panel(Settings settings) {
    this->show_overlay_panel(this->overlay_panels.settings_panel);
    this->overlay_panels.settings_panel->set_current_settings_panel(settings);
}

void MainFrame::show_connection_status_panel() {
    this->show_overlay_panel(this->overlay_panels.connection_status_panel);
}

void MainFrame::show_generate_profile_panel() {
    this->show_overlay_panel(this->overlay_panels.generate_profile_panel);
}

void MainFrame::show_import_legacy_profile_panel() {
    this->show_overlay_panel(this->overlay_panels.import_legacy_profile_panel);
}

void MainFrame::hide_overlay_panel() {
    assert(this->main_panels.current != nullptr);
    assert(this->overlay_panels.current != nullptr);

    this->overlay_panels.current->Show(false);
    this->overlay_panels.current = nullptr;

    this->main_panels.current->Show(true);
    this->Layout();
}

void MainFrame::show_main_panel(wxPanel* panel) {
    if (this->main_panels.current) {
        this->main_panels.current->Show(false);
    }
    this->main_panels.current = panel;
    this->main_panels.current->Show(true);
    this->Layout();
}

void MainFrame::show_overlay_panel(wxPanel* panel) {
    assert(this->main_panels.current != nullptr);
    this->main_panels.current->Show(false);

    if (this->overlay_panels.current != nullptr) {
        this->overlay_panels.current->Show(false);
    }

    this->overlay_panels.current = panel;
    this->overlay_panels.current->Show(true);
    this->Layout();
}

void MainFrame::open_profile(const wxString& profile_path) {
    this->get_sessions_notebook_panel_mut().open_session(profile_path);
    this->show_sessions_notebook_panel();
}

void MainFrame::enable_settings_menu_item(bool enable) {
    this->settings_menu_item->Enable(enable);
}

//
// Widget initialisation
//

void MainFrame::setup_menubar() {
    // create MenuBar
    auto menu_bar = new wxMenuBar();

    // create Profile menu
    auto profile_menu = new wxMenu();
    auto new_profile_menu_item =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::new_profile());
    profile_menu
        ->Bind(wxEVT_MENU, &MainFrame::on_new_profile, this, new_profile_menu_item->GetId());
    auto import_profile_menu_item = profile_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::import_profile()
    );
    profile_menu
        ->Bind(wxEVT_MENU, &MainFrame::on_import_legacy, this, import_profile_menu_item->GetId());
    auto open_profile_menu_item =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::open_profile());
    profile_menu
        ->Bind(wxEVT_MENU, &MainFrame::on_open_profile, this, open_profile_menu_item->GetId());
    auto save_profile_as_menu_item = profile_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::save_profile_as()
    );
    auto edit_profile_menu_item =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::edit_profile());

    profile_menu->AppendSeparator();

    auto close_profile_menu_item =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::close_profile());
    profile_menu
        ->Bind(wxEVT_MENU, &MainFrame::on_close_profile, this, close_profile_menu_item->GetId());
    auto logout_menu_item =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::logout());

    profile_menu->AppendSeparator();

    auto copy_user_id_menu_item =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::copy_user_id());
    auto set_visibility_menu = new wxMenu();
    auto visible_menu_item = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::visible()
    );
    auto restricted_menu_item = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::restricted()
    );
    auto hidden_menu_item = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::hidden()
    );
    auto offline_menu_item = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::offline()
    );

    auto set_visibility = profile_menu->AppendSubMenu(
        set_visibility_menu,
        Strings::MainFrame::MenuBar::Menu::Profile::set_visibility()
    );

    profile_menu->AppendSeparator();

    auto logout_all_menu_item =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::logout_all());
    auto quit_menu_item =
        profile_menu->Append(wxID_EXIT, Strings::MainFrame::MenuBar::Menu::Profile::quit());

    // create Contacts menu
    auto contacts_menu = new wxMenu();
    auto add_contact_menu_item =
        contacts_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Contacts::add_contact());
    auto delete_contact_menu_item = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::delete_contact()
    );
    auto connect_contact_menu_item = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::connect_contact()
    );
    auto disconnect_contact_menu_item = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::disconnect_contact()
    );
    auto block_contact_menu_item = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::block_contact()
    );
    auto unblock_contact_menu_item = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::unblock_contact()
    );

    // create Chat menu
    auto chat_menu = new wxMenu();
    auto export_logs_menu_item =
        chat_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Chat::export_logs());
    auto delete_logs_menu_item =
        chat_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Chat::delete_logs());

    // create Tools menu
    auto tools_menu = new wxMenu();
    auto file_transfers_menu_item =
        tools_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Tools::file_transfers());
    tools_menu->Bind(wxEVT_MENU, &MainFrame::on_file_transfers, this, file_transfers_menu_item->GetId());
    auto tor_logs_menu_item =
        tools_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Tools::tor_logs());
    tools_menu->Bind(wxEVT_MENU, &MainFrame::on_tor_logs, this, tor_logs_menu_item->GetId());
    this->settings_menu_item =
        tools_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Tools::settings());
    tools_menu->Bind(wxEVT_MENU, &MainFrame::on_settings, this, this->settings_menu_item->GetId());

    // create Help menu
    auto help_menu = new wxMenu();
    auto manual_menu_item =
        help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::manual());
    auto changelog_menu_item =
        help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::changelog());
    auto licenses_menu_item =
        help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::licenses());
    help_menu->AppendSeparator();
    auto check_for_updates_menu_item =
        help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::check_for_updates());
    auto about_menu_item =
        help_menu->Append(wxID_ABOUT, Strings::MainFrame::MenuBar::Menu::Help::about());

    // add Menus to MenUBar
    menu_bar->Append(profile_menu, Strings::MainFrame::MenuBar::profile());
    menu_bar->Append(contacts_menu, Strings::MainFrame::MenuBar::contacts());
    menu_bar->Append(chat_menu, Strings::MainFrame::MenuBar::chat());
    menu_bar->Append(tools_menu, Strings::MainFrame::MenuBar::tools());
    menu_bar->Append(help_menu, Strings::MainFrame::MenuBar::help());

    this->SetMenuBar(menu_bar);
}

void MainFrame::setup_main_panels(wxBoxSizer* sizer) {
    auto& main_panels = this->main_panels;

    auto bootstrap_panel = new BootstrapPanel(this);
    bootstrap_panel->show_disconnected();
    sizer->Add(bootstrap_panel, 1, wxEXPAND);
    bootstrap_panel->Hide();
    main_panels.bootstrap_panel = bootstrap_panel;

    auto sessions_notebook_panel = new SessionsNotebook(this);
    sizer->Add(sessions_notebook_panel, 1, wxEXPAND);
    sessions_notebook_panel->Hide();
    main_panels.sessions_notebook_panel = sessions_notebook_panel;
}

void MainFrame::setup_overlay_panels(wxBoxSizer* sizer) {
    auto& overlay_panels = this->overlay_panels;

    auto file_transfers_panel = new FileTransfersPanel(this);
    sizer->Add(file_transfers_panel, 1, wxEXPAND);
    file_transfers_panel->Hide();
    overlay_panels.file_transfers_panel = file_transfers_panel;

    // todo: ensure exiting these panels is consistent (e.g. with an 'Ok' button)
    auto settings_panel = new SettingsPanel(this);
    sizer->Add(settings_panel, 1, wxEXPAND);
    settings_panel->Hide();
    overlay_panels.settings_panel = settings_panel;

    auto connection_status_panel = new ConnectionStatusPanel(this);
    sizer->Add(connection_status_panel, 1, wxEXPAND);
    connection_status_panel->Hide();
    overlay_panels.connection_status_panel = connection_status_panel;

    auto generate_profile_panel = new NewProfilePanel(this, NewProfile::Generate);
    sizer->Add(generate_profile_panel, 1, wxEXPAND);
    generate_profile_panel->Hide();
    overlay_panels.generate_profile_panel = generate_profile_panel;

    auto import_legacy_profile_panel = new NewProfilePanel(this, NewProfile::ImportLegacy);
    sizer->Add(import_legacy_profile_panel, 1, wxEXPAND);
    import_legacy_profile_panel->Hide();
    overlay_panels.import_legacy_profile_panel = import_legacy_profile_panel;
}

//
// Event Handlers
//

void MainFrame::on_open_profile(wxCommandEvent&) {
    wxFileDialog open_profile_dialog(
        this,
        Strings::ConnectedPanel::open_profile_file_dialog_title(),
        Paths::home().GetAbsolutePath(),
        "",
        Strings::ConnectedPanel::profile_file_dialog_wildcard(),
        wxFD_OPEN | wxFD_FILE_MUST_EXIST
    );

    if (open_profile_dialog.ShowModal() == wxID_OK) {
        const auto profile_path = open_profile_dialog.GetPath();
        this->open_profile(profile_path);
    }
}

void MainFrame::on_new_profile(wxCommandEvent&) {
    this->show_generate_profile_panel();
}

void MainFrame::on_import_legacy(wxCommandEvent&) {
    this->show_import_legacy_profile_panel();
}

void MainFrame::on_close_profile(wxCommandEvent&) {
    this->get_sessions_notebook_panel_mut().close_focused_session();
}

void MainFrame::on_file_transfers(wxCommandEvent&) {
    this->show_file_transfers_panel();
}

void MainFrame::on_settings(wxCommandEvent&) {
    this->show_settings_panel(Settings::General);
}

void MainFrame::on_tor_logs(wxCommandEvent&) {
    this->show_connection_status_panel();
}
