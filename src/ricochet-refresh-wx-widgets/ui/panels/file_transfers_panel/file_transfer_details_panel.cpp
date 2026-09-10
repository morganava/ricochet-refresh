#include "file_transfer_details_panel.hpp"

#include "ui/fonts.hpp"
#include "ui/metrics.hpp"

FileTransferDetailsPanel::FileTransferDetailsPanel(wxWindow* parent) : wxPanel(parent) {
    auto root_h_sizer = new wxBoxSizer(wxHORIZONTAL);

    if (auto left_v_sizer = new wxBoxSizer(wxVERTICAL)) {
        auto heading = new wxStaticText(this, wxID_ANY, "Activity");
        heading->SetFont(Fonts::heading_font());
        left_v_sizer
            ->Add(heading, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);

        auto key_value_sizer = new wxBoxSizer(wxVERTICAL);
        left_v_sizer->Add(key_value_sizer, 0, wxEXPAND | wxLEFT, Metrics::PADDING_LARGE);

        // Time Elapsed
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                "Time Elapsed: ",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            auto value = new wxStaticText(
                this,
                wxID_ANY,
                "2m 36s",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_END
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Transferred
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                "Transferred: ",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            auto value = new wxStaticText(
                this,
                wxID_ANY,
                "3.55 MiB",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_END
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(value, 1, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Average Speed
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                "Average Speed: ",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            auto value = new wxStaticText(
                this,
                wxID_ANY,
                "22.8 KiB/s",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Status
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                "Status: ",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            auto value = new wxStaticText(
                this,
                wxID_ANY,
                "Downloading",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_END
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND);
        }
        key_value_sizer->AddSpacer(Metrics::VERTICAL_PADDING_MEDIUM);
        root_h_sizer->Add(left_v_sizer, 1, wxEXPAND, 0);
    }

    if (auto right_v_sizer = new wxBoxSizer(wxVERTICAL)) {
        auto heading = new wxStaticText(this, wxID_ANY, "Details");
        heading->SetFont(Fonts::heading_font());
        right_v_sizer
            ->Add(heading, 0, wxEXPAND | wxLEFT | wxRIGHT | wxBOTTOM, Metrics::PADDING_MEDIUM);

        auto key_value_sizer = new wxBoxSizer(wxVERTICAL);
        right_v_sizer->Add(key_value_sizer, 0, wxEXPAND | wxLEFT, Metrics::PADDING_LARGE);

        // Location
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                "Location: ",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            auto value = new wxStaticText(
                this,
                wxID_ANY,
                "/home/alice/Documents/Programming with wxWidgets.pdf",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_MIDDLE
            );
            auto folder_button = new wxBitmapButton(
                this,
                wxID_OPEN,
                wxArtProvider::GetBitmap(wxART_FOLDER, wxART_BUTTON)
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(value, 1, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(folder_button, 0, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Size
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                "Size: ",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            auto value = new wxStaticText(
                this,
                wxID_ANY,
                "7.1 MiB",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Hash
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                "Hash: ",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            auto value = new wxStaticText(
                this,
                wxID_ANY,
                "5e63d8f1d01174aff5c76b4f38fcfda05a1ee329a7f628862c20eb1750accdaf",
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_MIDDLE
            );
            auto copy_button = new wxBitmapButton(
                this,
                wxID_COPY,
                wxArtProvider::GetBitmap(wxART_COPY, wxART_BUTTON)
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(value, 1, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(copy_button, 0, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND);
        }
        key_value_sizer->AddSpacer(Metrics::VERTICAL_PADDING_MEDIUM);
        root_h_sizer->Add(right_v_sizer, 2, wxEXPAND, 0);
    }

    this->SetSizerAndFit(root_h_sizer);
}
