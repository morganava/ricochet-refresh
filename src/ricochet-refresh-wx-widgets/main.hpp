#pragma once

class RicochetRefresh: public wxApp {
public:
    bool OnInit() override;
    void OnInitCmdLine(wxCmdLineParser& parser) override;
    bool OnCmdLineParsed(wxCmdLineParser& parser) override;

private:
    class MainFrame* main_frame = nullptr;

    std::unique_ptr<tego_context> context;

    // the config we will load rather than the default
    std::optional<std::filesystem::path> config_to_load = std::nullopt;

    // the initial profiles to load after bootstrap
    std::optional<std::vector<std::filesystem::path>> profiles_to_load = std::nullopt;
};
