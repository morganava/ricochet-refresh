#include "file_transfer_row.hpp"

#include "enums.hpp"
#include "ffi.hpp"

//
// FileTransferRow
//

FileTransferRow::FileTransferRow(
    wxString filename,
    uint64_t size,
    FileTransferDirection direction,
    wxString sender,
    wxString receiver,
    wxDateTime now
) :
    filename(filename),
    size(size),
    bytes_transferred(0),
    direction(direction),
    sender(sender),
    receiver(receiver),
    speed(0.0),
    eta({}),
    date_added(now),
    speed_calculator(into_tego_time(now)) {}

template<typename T>
Ordering compare(const T& a, const T& b) {
    if (a < b) {
        return Ordering::Less;
    } else if (a > b) {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

template<>
Ordering compare<wxString>(const wxString& a, const wxString& b) {
    if (auto cmp = a.CmpNoCase(b); cmp < 0) {
        return Ordering::Less;
    } else if (cmp > 0) {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

template<>
Ordering compare<wxDateTime>(const wxDateTime& a, const wxDateTime& b) {
    if (a.IsEarlierThan(b)) {
        return Ordering::Less;
    } else if (a.IsLaterThan(b)) {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

Ordering FileTransferRow::compare(
    const FileTransferRow& row1,
    const FileTransferRow& row2,
    FileTransfersPanelListColumn column
) {
    switch (column) {
        case FileTransfersPanelListColumn::Filename:
            return ::compare(row1.filename, row2.filename);
        case FileTransfersPanelListColumn::Size:
            return ::compare(row1.size, row2.size);
        case FileTransfersPanelListColumn::Status:
            return ::compare(
                static_cast<double>(row1.bytes_transferred) / static_cast<double>(row1.size),
                static_cast<double>(row2.bytes_transferred) / static_cast<double>(row2.size)
            );
        case FileTransfersPanelListColumn::Receiver:
            return ::compare(row1.receiver, row2.receiver);
        case FileTransfersPanelListColumn::Sender:
            return ::compare(row1.sender, row2.sender);
        case FileTransfersPanelListColumn::Speed:
            return ::compare(row1.speed, row2.speed);
        case FileTransfersPanelListColumn::ETA:
            return ::compare(row1.eta, row2.eta);
        case FileTransfersPanelListColumn::DateAdded:
            return ::compare(row1.date_added, row2.date_added);
    }
    tego::panic(fmt::format("Unknown column type: {}", static_cast<unsigned int>(column)));
}

void FileTransferRow::update(tego_time now) {
    this->speed = this->speed_calculator.get_average_speed(now);

    if (this->speed > 0.0 && this->speed < std::numeric_limits<double>::infinity()) {
        const auto remaining = this->size - this->bytes_transferred;
        const auto eta_milliseconds =
            static_cast<uint64_t>(static_cast<double>(remaining) / this->speed * 1000.0);
        this->eta = wxTimeSpan::Milliseconds(eta_milliseconds);
    } else {
        this->eta = std::nullopt;
    }
}

void FileTransferRow::update_bytes_transferred(uint64_t bytes_transferred, tego_time now) {
    TEGO_PANIC_IF(bytes_transferred < this->bytes_transferred);
    this->bytes_transferred = bytes_transferred;
    this->speed_calculator.add_record(bytes_transferred, now);
}
