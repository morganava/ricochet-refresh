#pragma once

class RicochetRefresh: public wxApp {
public:
    bool OnInit() override;

private:
    class MainFrame* main_frame = nullptr;
};