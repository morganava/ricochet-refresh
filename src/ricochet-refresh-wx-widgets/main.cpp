#include "main.hpp"

#include "locale.hpp"
#include "strings.hpp"
#include "ui/main_frame.hpp"

wxIMPLEMENT_APP(RicochetRefresh);

bool RicochetRefresh::OnInit() {
    if (!wxApp::OnInit()) {
        return false;
    }

    Locale::init();

    auto main_frame = new MainFrame();
    main_frame->Show(true);
    this->main_frame = main_frame;

    return true;
}
