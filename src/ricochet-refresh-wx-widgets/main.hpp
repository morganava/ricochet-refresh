#pragma once

class RicochetRefresh: public wxApp {
public:
    bool OnInit() override;
    void OnInitCmdLine(wxCmdLineParser& parser) override;
    bool OnCmdLineParsed(wxCmdLineParser& parser) override;

    class MainFrame& get_main_frame() {
        return *this->main_frame;
    }

    const tego_context& get_context() const {
        return *this->context.get();
    }

    tego_context& get_context_mut() {
        return *this->context.get();
    }

    const tego_settings& get_settings() const {
        return *this->settings.get();
    }

    tego_settings& get_settings_mut() {
        return *this->settings.get();
    }

private:
    void init_settings();

    class MainFrame* main_frame = nullptr;

    std::unique_ptr<tego_context> context;
    std::unique_ptr<tego_settings> settings;

    // the config we will load rather than the default
    std::optional<std::filesystem::path> config_to_load = std::nullopt;

    // the initial profiles to load after bootstrap
    std::optional<std::vector<std::filesystem::path>> profiles_to_load = std::nullopt;
};

wxDECLARE_APP(RicochetRefresh);
