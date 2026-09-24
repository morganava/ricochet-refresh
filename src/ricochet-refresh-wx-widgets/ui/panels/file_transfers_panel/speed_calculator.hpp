#pragma once
// number of bytes
typedef uint64_t tego_file_size;
// timestamp in milliseconds since unix epoch
typedef uint64_t tego_time;

// SAMPLE_COUNT: max number of samples to collect
// WINDOW_MS:  max size of our time window to consider in milliseconds
template<size_t MAX_SAMPLE_COUNT, size_t MAX_WINDOW_MS>
class AverageSpeedCalculator {
public:
    AverageSpeedCalculator(tego_time start_time) {
        this->record_buffer[0] = std::make_tuple(0, start_time);
    }

    // timestamp and total_bytes_transferred each MUST increase monotonically with each new record
    void add_record(tego_file_size total_bytes_transferred, tego_time timestamp) {
        // handle timestamp edgecase
        if (this->record_count > 0) {
            const auto& head_record = this->record_buffer[this->head];
            const auto& head_timestamp = std::get<TIMESTAMP>(head_record);
            auto& head_total_transferred = std::get<TOTAL_TRANSFERRED>(head_record);
            // ensure this record is not for the past
            TEGO_PANIC_IF(head_timestamp > timestamp);
            if (head_timestamp == timestamp) {
                // ensure this record does not decrease the bytes transferred
                TEGO_PANIC_IF(head_total_transferred > total_bytes_transferred);
                // simply update head record with new value if greater
                head_total_transferred = total_bytes_transferred;
                return;
            }
        }

        // handle circular buffer overwrite
        // if buffer is full, we are about to overwrite the oldest record (tail)
        // subtract that record's delta from our sliding window sum
        if (this->record_count < MAX_SAMPLE_COUNT) {
            this->record_count += 1;
        }

        // advance head
        this->head = (this->head + 1) % MAX_SAMPLE_COUNT;

        // store the record
        this->record_buffer[head] = std::make_tuple(total_bytes_transferred, timestamp);
    }

    // timestamp: the current time (i.e. now)
    void evict_expired_records(tego_time timestamp) {
        // time-based record eviction
        // remove records from the tail that have fallen outside the WINDOW_MS
        while (this->record_count > 0) {
            const auto tail = this->tail();
            const auto tail_timestamp = std::get<TIMESTAMP>(this->record_buffer[tail]);

            if ((timestamp - tail_timestamp) > MAX_WINDOW_MS) {
                // record is too old so we evict
                record_count -= 1;
            } else {
                // oldest remaining record is still within the window
                return;
            }
        }
    }

    // returns average transfer speed in bytes / second
    double get_average_speed(tego_time now) {
        // evict records older than MAX_WINDOW_MS
        this->evict_expired_records(now);

        // we need at least 2 records to calculate the average transfer speed
        if (this->record_count < 2) {
            return 0.0;
        }

        // get time window

        auto& tail_record = this->record_buffer[this->tail()];

        auto head_timestamp = now;
        auto tail_timestamp = std::get<TIMESTAMP>(tail_record);

        auto ms_time_delta = head_timestamp - tail_timestamp;
        // records coming in too fast for now to calculate speed
        if (ms_time_delta == 0) {
            return std::numeric_limits<double>::infinity();
        }

        // get bytes transferred within window

        auto& head_record = this->record_buffer[this->head];

        auto head_total_transferred = std::get<TOTAL_TRANSFERRED>(head_record);
        auto tail_total_transferred = std::get<TOTAL_TRANSFERRED>(tail_record);

        auto bytes_delta = head_total_transferred - tail_total_transferred;
        auto bytes_per_ms = static_cast<double>(bytes_delta) / static_cast<double>(ms_time_delta);
        constexpr static double MS_PER_SECOND = 1000.0;
        auto bytes_per_s = bytes_per_ms * MS_PER_SECOND;

        return bytes_per_s;
    }

private:
    // first value is the number of additional bytes transferred since last record
    // second value is timestamp record was received
    constexpr static size_t TOTAL_TRANSFERRED = 0;
    constexpr static size_t TIMESTAMP = 1;
    typedef std::tuple<tego_file_size, tego_time> record;
    std::array<record, MAX_SAMPLE_COUNT> record_buffer = {};

    // the number of active records
    // starts at 1 becuase we always add an initial no-bytes transferred record in constructor
    size_t record_count = 1;
    // index of newest record
    size_t head = 0;

    // index of oldest record
    size_t tail() const {
        return (head + (MAX_SAMPLE_COUNT - record_count) + 1) % MAX_SAMPLE_COUNT;
    }
};
