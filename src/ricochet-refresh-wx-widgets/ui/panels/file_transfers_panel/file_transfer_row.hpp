#pragma once

#ifndef SPEED_CALCULATOR_H
    #include "ui/panels/file_transfers_panel/speed_calculator.hpp"
#endif // SPEED_CALCUALTOR_H

enum class FileTransferDirection;
enum class Ordering;
enum class FileTransfersPanelListColumn : unsigned int;

class FileTransferRow {
public:
    FileTransferRow(
        wxString filename,
        uint64_t size,
        FileTransferDirection direction,
        wxString sender,
        wxString receiver,
        wxDateTime now
    );

    //
    // Getters
    //
    const wxString& get_filename() const {
        return this->filename;
    }

    uint64_t get_size() const {
        return this->size;
    }

    uint64_t get_bytes_transferred() const {
        return this->bytes_transferred;
    }

    FileTransferDirection get_direction() const {
        return this->direction;
    }

    double get_percent_progress() const {
        // special-case completion just in case that static cast + division doesn't work out to 100.0 exactly
        // it *should* but sometimes floating point is weird vOv
        if (this->bytes_transferred == this->size) {
            return 100.0;
        }
        return static_cast<double>(this->bytes_transferred) / static_cast<double>(this->size)
            * 100.0;
    }

    const wxString& get_sender() const {
        return this->sender;
    }

    const wxString& get_receiver() const {
        return this->receiver;
    }

    double get_speed() const {
        return this->speed;
    }

    const std::optional<wxTimeSpan>& get_eta() const {
        return this->eta;
    }

    const wxDateTime& get_date_added() const {
        return this->date_added;
    }

    //
    // Compare for sorting
    //
    static Ordering compare(
        const FileTransferRow& row1,
        const FileTransferRow& row2,
        FileTransfersPanelListColumn column
    );

    //
    // Update methods
    //

    void update(tego_time now);
    void update_bytes_transferred(uint64_t bytes_transferred, tego_time now);

private:
    wxString filename;
    uint64_t size;
    uint64_t bytes_transferred;
    FileTransferDirection direction;
    wxString sender;
    wxString receiver;
    double speed; // bytes per second
    std::optional<wxTimeSpan> eta;
    wxDateTime date_added;

    // cppcheck-suppress unusedStructMember
    constexpr static size_t SAMPLES = 32;
    // cppcheck-suppress unusedStructMember
    constexpr static size_t TIME_DELTA_MS = 5000; // 5 seconds

    AverageSpeedCalculator<SAMPLES, TIME_DELTA_MS> speed_calculator;
};
