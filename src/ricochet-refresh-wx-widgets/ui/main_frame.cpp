#include "main_frame.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/bootstrap_panel.hpp"
#include "ui/panels/connection_status_panel.hpp"
#include "ui/panels/conversations_panel.hpp"
#include "ui/panels/settings_panel.hpp"
#include "ui/widgets/wrapped_static_text.hpp"

MainFrame::MainFrame() : wxFrame(nullptr, wxID_ANY, Strings::MainFrame::title()) {
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

void MainFrame::show_profile_notebook_panel() {
    // todo
}

void MainFrame::show_settings_panel(Settings settings) {
    this->show_overlay_panel(this->overlay_panels.settings_panel);
    this->overlay_panels.settings_panel->set_current_settings_panel(settings);
}

void MainFrame::show_connection_status_panel() {
    this->show_overlay_panel(this->overlay_panels.connection_status_panel);
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
    assert(this->main_panels.current != panel);
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

//
// Widget initialisation
//

void MainFrame::setup_menubar() {
    // create MenuBar
    auto menu_bar = new wxMenuBar();

    // create Profile menu
    auto profile_menu = new wxMenu();
    auto new_profile =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::new_profile());
    auto open_profile =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::open_profile());
    auto save_profile_as = profile_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::save_profile_as()
    );
    auto edit_profile =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::edit_profile());

    profile_menu->AppendSeparator();

    auto close_profile =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::close_profile());
    auto logout =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::logout());

    profile_menu->AppendSeparator();

    auto copy_user_id =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::copy_user_id());
    auto set_visibility_menu = new wxMenu();
    auto visible = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::visible()
    );
    auto restricted = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::restricted()
    );
    auto hidden = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::hidden()
    );
    auto offline = set_visibility_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Profile::SetVisibility::offline()
    );

    auto set_visibility = profile_menu->AppendSubMenu(
        set_visibility_menu,
        Strings::MainFrame::MenuBar::Menu::Profile::set_visibility()
    );

    profile_menu->AppendSeparator();

    auto logout_all =
        profile_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Profile::logout_all());
    auto _quit =
        profile_menu->Append(wxID_EXIT, Strings::MainFrame::MenuBar::Menu::Profile::quit());

    // create Contacts menu
    auto contacts_menu = new wxMenu();
    auto add_contact =
        contacts_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Contacts::add_contact());
    auto delete_contact = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::delete_contact()
    );
    auto connect_contact = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::connect_contact()
    );
    auto disconnect_contact = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::disconnect_contact()
    );
    auto block_contact = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::block_contact()
    );
    auto unblock_contact = contacts_menu->Append(
        wxID_ANY,
        Strings::MainFrame::MenuBar::Menu::Contacts::unblock_contact()
    );

    // create Chat menu
    auto chat_menu = new wxMenu();
    auto export_logs =
        chat_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Chat::export_logs());
    auto delete_logs =
        chat_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Chat::delete_logs());

    // create Tools menu
    auto tools_menu = new wxMenu();
    auto downloads =
        tools_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Tools::downloads());
    auto tor_logs =
        tools_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Tools::tor_logs());
    auto settings =
        tools_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Tools::settings());

    // create Help menu
    auto help_menu = new wxMenu();
    auto manual = help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::manual());
    auto changelog =
        help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::changelog());
    auto licenses =
        help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::licenses());
    help_menu->AppendSeparator();
    auto check_for_updates =
        help_menu->Append(wxID_ANY, Strings::MainFrame::MenuBar::Menu::Help::check_for_updates());
    auto about = help_menu->Append(wxID_ABOUT, Strings::MainFrame::MenuBar::Menu::Help::about());

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

    // todo develop the profile ntebook panel
}

void MainFrame::setup_overlay_panels(wxBoxSizer* sizer) {
    auto& overlay_panels = this->overlay_panels;

    // todo: ensure exiting these panels is consistent (e.g. with an 'Ok' button)
    auto settings_panel = new SettingsPanel(this);
    sizer->Add(settings_panel, 1, wxEXPAND);
    settings_panel->Hide();
    overlay_panels.settings_panel = settings_panel;

    auto connection_status_panel = new ConnectionStatusPanel(
        this,
        Strings::ConnectionStatusPanel::bundled_client_string("tor", "0.4.8.21"),
        ConnectionStatus::Online
    );
    sizer->Add(connection_status_panel, 1, wxEXPAND);
    connection_status_panel->Hide();
    overlay_panels.connection_status_panel = connection_status_panel;
}
