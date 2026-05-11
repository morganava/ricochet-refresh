#include "main.hpp"

#include "locale.hpp"
#include "strings.hpp"
#include "ui/main_frame.hpp"

wxIMPLEMENT_APP(RicochetRefresh);

bool RicochetRefresh::OnInit() try {
    if (!wxApp::OnInit()) {
        return false;
    }

    Locale::init();

    auto main_frame = new MainFrame();
    main_frame->Show(true);
    this->main_frame = main_frame;

    tego_context_initialize(tego::out(this->context), tego::throw_on_error());

    return true;

} catch (std::exception& ex) {
    LOG_ERROR(ex.what());
    return false;
}
