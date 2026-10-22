#include "file_transfer_details_panel.hpp"

#include "enums.hpp"
#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/metrics.hpp"

FileTransferDetailsPanel::FileTransferDetailsPanel(wxWindow* parent) :
    wxPanel(parent),
    size(size),
    status(FileTransferStatus::Pending) {
    auto root_h_sizer = new wxBoxSizer(wxHORIZONTAL);

    if (auto left_v_sizer = new wxBoxSizer(wxVERTICAL)) {
        auto heading = new wxStaticText(
            this,
            wxID_ANY,
            Strings::FileTransfersPanel::Details::activity_heading()
        );
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
                Strings::FileTransfersPanel::Details::time_elapsed_label(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            this->time_elapsed_value = new wxStaticText(
                this,
                wxID_ANY,
                wxString(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_END | wxST_NO_AUTORESIZE
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(this->time_elapsed_value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Transferred
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                Strings::FileTransfersPanel::Details::transferred_label(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            this->transferred_value = new wxStaticText(
                this,
                wxID_ANY,
                wxString(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_END | wxST_NO_AUTORESIZE
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(
                this->transferred_value,
                1,
                wxEXPAND | wxRIGHT,
                Metrics::HORIZONTAL_PADDING_SMALL
            );
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Average Speed
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                Strings::FileTransfersPanel::Details::average_speed_label(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            this->average_speed_value = new wxStaticText(
                this,
                wxID_ANY,
                wxString(""),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_NO_AUTORESIZE
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(this->average_speed_value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Status
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                Strings::FileTransfersPanel::Details::status_label(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            this->status_value = new wxStaticText(
                this,
                wxID_ANY,
                wxString(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_END | wxST_NO_AUTORESIZE
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(this->status_value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND);
        }
        key_value_sizer->AddSpacer(Metrics::VERTICAL_PADDING_MEDIUM);
        root_h_sizer->Add(left_v_sizer, 1, wxEXPAND, 0);
    }

    if (auto right_v_sizer = new wxBoxSizer(wxVERTICAL)) {
        auto heading = new wxStaticText(
            this,
            wxID_ANY,
            Strings::FileTransfersPanel::Details::details_heading()
        );
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
                Strings::FileTransfersPanel::Details::location_label(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            this->location_value = new wxStaticText(
                this,
                wxID_ANY,
                wxString(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT | wxST_ELLIPSIZE_MIDDLE | wxST_NO_AUTORESIZE
            );
            auto folder_button = new wxBitmapButton(
                this,
                wxID_OPEN,
                wxArtProvider::GetBitmap(wxART_FOLDER, wxART_BUTTON)
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(
                this->location_value,
                1,
                wxEXPAND | wxRIGHT,
                Metrics::HORIZONTAL_PADDING_SMALL
            );
            h_sizer->Add(folder_button, 0, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Size
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                Strings::FileTransfersPanel::Details::size_label(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            this->size_value = new wxStaticText(
                this,
                wxID_ANY,
                wxString(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_RIGHT
            );
            h_sizer->Add(label, 0, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(this->size_value, 1, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND | wxBOTTOM, Metrics::VERTICAL_PADDING_MEDIUM);
        }
        // Hash
        if (auto h_sizer = new wxBoxSizer(wxHORIZONTAL)) {
            auto label = new wxStaticText(
                this,
                wxID_ANY,
                Strings::FileTransfersPanel::Details::hash_label(),
                wxDefaultPosition,
                wxDefaultSize,
                wxALIGN_LEFT | wxST_ELLIPSIZE_END
            );
            this->hash_value = new wxStaticText(
                this,
                wxID_ANY,
                wxString(),
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
            h_sizer
                ->Add(this->hash_value, 1, wxEXPAND | wxRIGHT, Metrics::HORIZONTAL_PADDING_SMALL);
            h_sizer->Add(copy_button, 0, wxEXPAND);
            key_value_sizer->Add(h_sizer, 0, wxEXPAND);
        }
        key_value_sizer->AddSpacer(Metrics::VERTICAL_PADDING_MEDIUM);
        root_h_sizer->Add(right_v_sizer, 2, wxEXPAND, 0);
    }

    this->SetSizerAndFit(root_h_sizer);
}

FileTransferDetailsPanel::FileTransferDetailsPanel(
    wxWindow* parent,
    const wxString location,
    uint64_t size,
    const wxString& hash
) :
    FileTransferDetailsPanel(parent) {
    this->location_value->SetLabel(location);
    this->size_value->SetLabel(Strings::Common::file_size_formatted(size));
    this->hash_value->SetLabel(hash);
}

FileTransferDetailsPanel::FileTransferDetailsPanel(
    wxWindow* parent,
    uint64_t size,
    const wxString& hash
) :
    FileTransferDetailsPanel(parent) {
    this->size_value->SetLabel(Strings::Common::file_size_formatted(size));
    this->hash_value->SetLabel(hash);
}

void FileTransferDetailsPanel::update(const wxDateTime& now) {
    if (this->status == FileTransferStatus::Pending) {
        return;
    }

    if (this->transfer_start_time.has_value()) {
        // time elapsed
        const auto elapsed =
            (this->transfer_end_time.has_value() ? this->transfer_end_time.value() : now)
            - this->transfer_start_time.value();
        this->time_elapsed_value->SetLabel(Strings::Common::time_span_formatted(elapsed));

        // transferred
        if (this->bytes_transferred.has_value()) {
            const auto bytes_transferred = this->bytes_transferred.value();
            this->transferred_value->SetLabel(Strings::Common::file_size_formatted(bytes_transferred
            ));

            // average speed
            const auto average_speed =
                bytes_transferred / static_cast<uint64_t>(elapsed.GetSeconds().GetValue());
            this->average_speed_value->SetLabel(
                Strings::Common::transfer_speed_formatted(average_speed)
            );

            // status
            const auto percent_complete = 100.0 * static_cast<double>(*this->bytes_transferred)
                / static_cast<double>(this->size);
            this->status_value->SetLabel(
                Strings::FileTransfersPanel::status_formatted(this->status, percent_complete)
            );
        } else {
            this->status_value->SetLabel(
                Strings::FileTransfersPanel::status_formatted(this->status, 0.0)
            );
        }
    }
}

void FileTransferDetailsPanel::begin_pending_download(wxString location) {
    this->location_value->SetLabel(location);
}

void FileTransferDetailsPanel::update_transfer_progress(tego_file_size bytes_transferred) {
    this->bytes_transferred = bytes_transferred;
}

void FileTransferDetailsPanel::update_status(FileTransferStatus status) {
    switch (this->status) {
        case FileTransferStatus::Pending:
            switch (status) {
                case FileTransferStatus::Downloading:
                case FileTransferStatus::Uploading:
                    this->transfer_start_time = wxDateTime::Now();
                    break;
            }
            break;
        case FileTransferStatus::Downloading:
        case FileTransferStatus::Uploading:
            this->transfer_end_time = wxDateTime::Now();
            break;
        default:
            break;
    }
    this->status = status;
}

void FileTransferDetailsPanel::update_location(wxString location) {
    this->location_value->SetLabel(location);
}
