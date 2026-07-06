#pragma once
// C

// libtego
#include <tego/tego.h>

// C++

// standard library
#include <stdexcept>
#include <memory>
#include <utility>
#include <memory>
#include <type_traits>

// libtego
#include <tego/utilities.hpp>
#include <tego/logger.hpp>

namespace tego
{
    //
    // base-class for tego_error handlers
    //
    class do_on_error {
    public:
        ~do_on_error() {
            if (this->error != nullptr) {
                tego_error_delete(this->error);
            }
        }

        operator tego_error**()
        {
            return &error;
        }

    protected:
        std::optional<std::string> get_error_message() const {
            if (this->error) {
                return std::string(tego_error_get_message(this->error));
            } else {
                return std::nullopt;
            }
        }

        tego_error* error = nullptr;
    };

    //
    // converts tego_error** C style error handling to exceptions
    //
    class throw_on_error : public do_on_error {
    public:
        ~throw_on_error() noexcept(false) {
            if (const auto msg = this->get_error_message()) {
                std::runtime_error ex(*msg);
                throw ex;
            }
        }
    };

    //
    // only log errors to Error channel
    //
    class log_on_error : public do_on_error {
    public:
        ~log_on_error() {
            if (const auto msg = this->get_error_message()) {
                LOG_ERROR(*msg);
            }
        }
    };

    //
    // crash on error
    //
    class panic_on_error : public do_on_error {
    public:
        ~panic_on_error() {
            if (const auto msg = this->get_error_message()) {
                tego_panic(msg->data(), msg->length());
            }
        }
    };
}


// define deleters for using unique_ptr and shared_ptr with tego types

#define TEGO_DEFAULT_DELETE_IMPL(TYPE)\
namespace std {\
    template<> class default_delete<TYPE> {\
    public:\
        void operator()(TYPE* val) { TYPE##_delete(val); }\
    };\
}

TEGO_DEFAULT_DELETE_IMPL(tego_error)
TEGO_DEFAULT_DELETE_IMPL(tego_string)
TEGO_DEFAULT_DELETE_IMPL(tego_context)
TEGO_DEFAULT_DELETE_IMPL(tego_settings)
TEGO_DEFAULT_DELETE_IMPL(tego_ed25519_private_key)
TEGO_DEFAULT_DELETE_IMPL(tego_v3_onion_service_id)
TEGO_DEFAULT_DELETE_IMPL(tego_user_id)
TEGO_DEFAULT_DELETE_IMPL(tego_tor_config)
TEGO_DEFAULT_DELETE_IMPL(tego_bridge_config)
TEGO_DEFAULT_DELETE_IMPL(tego_proxy_config)
TEGO_DEFAULT_DELETE_IMPL(tego_firewall_config)
