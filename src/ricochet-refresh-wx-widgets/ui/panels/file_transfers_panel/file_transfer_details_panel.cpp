#include "file_transfer_details_panel.hpp"

#include "ui/fonts.hpp"
#include "ui/metrics.hpp"

FileTransferDetailsPanel::FileTransferDetailsPanel(wxWindow* parent) : wxPanel(parent) {
    auto h_sizer = new wxBoxSizer(wxHORIZONTAL);
    if (auto left_v_sizer = new wxBoxSizer(wxVERTICAL)) {
        auto heading = new wxStaticText(this, wxID_ANY, "Activity");
        heading->SetFont(Fonts::heading_font());
        left_v_sizer
            ->Add(heading, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);

        h_sizer->Add(left_v_sizer, 1, wxEXPAND, 0);
    }

    if (auto right_v_sizer = new wxBoxSizer(wxVERTICAL)) {
        auto heading = new wxStaticText(this, wxID_ANY, "Details");
        heading->SetFont(Fonts::heading_font());
        right_v_sizer
            ->Add(heading, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);
        h_sizer->Add(right_v_sizer, 2, wxEXPAND, 0);
    }

    this->SetSizerAndFit(h_sizer);
}
