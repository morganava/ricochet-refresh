#include "conversations_panel.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "strings.hpp"
#include "ui/events.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/chat_panel.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/contact_list_panel.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/message_entry_panel.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/user_status_panel.hpp"

ConversationsPanel::ConversationsPanel(wxWindow* parent, tego_session_handle session_handle) :
    wxSplitterWindow(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxSP_LIVE_UPDATE),
    session_handle(session_handle) {
    auto left_panel = new wxPanel(this);
    this->right_panel = new wxPanel(this);

    // Contacts List + User Status

    auto left_v_sizer = new wxBoxSizer(wxVERTICAL);

    this->contact_list_panel = new ContactListPanel(left_panel, session_handle);
    this->contact_list_panel->Bind(wxEVT_CONTACT_SELECTED, [this](const ContactSelectedEvent& evt) {
        this->select_contact(evt.get_contact_handle());
    });
    this->contact_list_panel->Bind(wxEVT_CONTACT_REMOVED, [this](ContactRemovedEvent& evt) {
        this->remove_contact(evt.get_contact_handle());
        evt.Skip();
    });
    auto user_status_panel = new UserStatusPanel(left_panel);

    left_v_sizer->Add(this->contact_list_panel, 1, wxEXPAND);
    left_v_sizer->Add(user_status_panel, 0, wxEXPAND);

    left_panel->SetSizer(left_v_sizer);
    left_panel->SetMinSize(wxSize(288, -1));

    // Conversation + Chat Entry

    this->right_v_sizer = new wxBoxSizer(wxVERTICAL);

    this->right_panel->SetSizer(this->right_v_sizer);
    this->right_panel->SetMinSize(wxSize(288, -1));

    // Layout

    this->SetMinimumPaneSize(32); // prevent dbl-click collapse
    this->SplitVertically(left_panel, this->right_panel, 288);
    this->SetSashGravity(0.0);
}

ConversationsPanel::~ConversationsPanel() {
    tego_context_end_session(
        &wxGetApp().get_context_mut(),
        this->session_handle,
        tego::panic_on_error()
    );
}

void ConversationsPanel::add_user(
    const tego_user_handle user_handle,
    const wxString& display_name,
    const wxBitmap& avatar,
    const ContactGroup contact_group
) {
    // add user to ContactList
    this->contact_list_panel->add_contact(user_handle, display_name, avatar, contact_group);

    // add ContactWidgets for chatting

    auto chat_panel = new ChatPanel(this->right_panel);
    // todo: load chat back-log from profile

    auto message_entry_panel = new MessageEntryPanel(this->right_panel);
    message_entry_panel->Bind(wxEVT_SEND_MESSAGE, [=, this](const SendMessageEvent& evt) {
        const auto& timestamp = evt.get_timestamp();
        const auto& text = evt.get_text();

        if (!this->handle_debug_command(text, user_handle)) {
            auto text_ts = into_tego_string(text);
            std::unique_ptr<tego_error> err;
            tego_message_id message_id;
            tego_context_send_message(
                &wxGetApp().get_context_mut(),
                this->session_handle,
                user_handle,
                text_ts.get(),
                &message_id,
                tego::out(err)
            );
            if (err) {
                // todo: on failure we should NOT erase the text in the chat box
                LOG_ERROR(err.get());
            } else {
                chat_panel->add_chat_message(timestamp, wxString("Me"), text);
            }
        } else {
            chat_panel->add_chat_message(timestamp, wxString("Me"), text);
        }
    });

    auto v_sizer = new wxBoxSizer(wxVERTICAL);
    v_sizer->Add(chat_panel, 1, wxEXPAND);
    v_sizer->Add(message_entry_panel, 0, wxEXPAND);

    this->right_v_sizer->Add(v_sizer, 1, wxEXPAND);
    this->right_v_sizer->Show(v_sizer, false);

    this->contact_widgets.insert(
        {user_handle, {display_name, v_sizer, chat_panel, message_entry_panel}}
    );
}

bool ConversationsPanel::handle_debug_command(
    const wxString& cmd,
    const tego_session_handle user_handle
) {
    auto context = &wxGetApp().get_context_mut();
    auto tokens = wxSplit(wxString(cmd).Trim(true).Trim(false), ' ');
    if (tokens.IsEmpty()) {
        return false;
    } else {
        auto command = tokens[0];
        const auto token_count = tokens.GetCount();
        if (token_count == 1) {
            if (command == "/accept_contact_request") {
                tego_context_acknowledge_chat_request(
                    context,
                    this->session_handle,
                    user_handle,
                    tego_chat_acknowledge_accept,
                    tego::log_on_error()
                );
                return true;
            } else if (command == "/reject_contact_request") {
                tego_context_acknowledge_chat_request(
                    context,
                    this->session_handle,
                    user_handle,
                    tego_chat_acknowledge_reject,
                    tego::log_on_error()
                );
                return true;
            } else if (command == "/forget_user") {
                tego_context_forget_user(
                    context,
                    this->session_handle,
                    user_handle,
                    tego::log_on_error()
                );
                return true;
            }
        } else if (token_count == 2) {
            if (command == "/send_transfer") {
                auto path = into_tego_string(tokens[1]);
                tego_file_transfer_id file_transfer_id;
                tego_file_size file_size;
                tego_context_send_file_transfer_request(
                    context,
                    this->session_handle,
                    user_handle,
                    path.get(),
                    &file_transfer_id,
                    &file_size,
                    tego::log_on_error()
                );
                return true;
            } else if (command == "/reject_transfer") {
                wxULongLong_t file_transfer_id;
                if (tokens[1].ToULongLong(&file_transfer_id)) {
                    tego_context_respond_file_transfer_request(
                        context,
                        this->session_handle,
                        user_handle,
                        static_cast<tego_file_transfer_id>(file_transfer_id),
                        tego_file_transfer_response_reject,
                        nullptr,
                        tego::log_on_error()
                    );
                    return true;
                }
            } else if (command == "/cancel_transfer") {
                wxULongLong_t file_transfer_id;
                if (tokens[1].ToULongLong(&file_transfer_id)) {
                    tego_context_cancel_file_transfer(
                        context,
                        this->session_handle,
                        user_handle,
                        static_cast<tego_file_transfer_id>(file_transfer_id),
                        tego::log_on_error()
                    );
                    return true;
                }
            }
        } else if (token_count == 3) {
            if (command == "/accept_transfer") {
                wxULongLong_t file_transfer_id;
                if (tokens[1].ToULongLong(&file_transfer_id)) {
                    auto path = into_tego_string(tokens[2]);
                    tego_context_respond_file_transfer_request(
                        context,
                        this->session_handle,
                        user_handle,
                        static_cast<tego_file_transfer_id>(file_transfer_id),
                        tego_file_transfer_response_accept,
                        path.get(),
                        tego::log_on_error()
                    );
                    return true;
                }
            } else if (command == "/add_contact") {
                auto service_id_string = tokens[1];
                std::unique_ptr<tego_v3_onion_service_id> service_id;
                std::unique_ptr<tego_error> error;

                tego_v3_onion_service_id_from_string(
                    tego::out(service_id),
                    into_tego_string(service_id_string).get(),
                    tego::out(error)
                );
                if (error) {
                    LOG_ERROR(error);
                } else {
                    auto pet_name = tokens[2];
                    tego_user_handle new_user_handle;
                    tego_context_send_chat_request(
                        context,
                        this->session_handle,
                        service_id.get(),
                        into_tego_string(pet_name).get(),
                        into_tego_string("Please add me!").get(),
                        &new_user_handle,
                        tego::log_on_error()
                    );
                }
            }
        }
    }

    return false;
}

void ConversationsPanel::change_user_contact_group(
    const tego_user_handle user_handle,
    const ContactGroup contact_group
) {
    this->contact_list_panel->move_contact(user_handle, contact_group);
}

void ConversationsPanel::receive_message(
    const tego_user_handle recipient,
    const wxDateTime& timestamp,
    const wxString& message
) {
    if (auto it = this->contact_widgets.find(recipient); it != this->contact_widgets.end()) {
        auto& contact_widgets = it->second;

        contact_widgets.chat_panel
            ->add_chat_message(timestamp, contact_widgets.display_name, message);
    }
}

void ConversationsPanel::select_contact(const std::optional<tego_user_handle> contact_handle) {
    // hide everything
    this->right_v_sizer->ShowItems(false);
    if (contact_handle) {
        if (auto it = this->contact_widgets.find(*contact_handle);
            it != this->contact_widgets.end()) {
            // show contact's widgets
            const auto& contact_widgets = it->second;
            this->right_v_sizer->Show(contact_widgets.v_sizer, true);
            this->right_v_sizer->Layout();
        }
    }
}

void ConversationsPanel::remove_contact(const tego_user_handle contact_handle) {
    if (auto it = this->contact_widgets.find(contact_handle); it != this->contact_widgets.end()) {
        auto& v_sizer = it->second.v_sizer;

        // remove and delete children
        v_sizer->Clear(true);
        this->right_v_sizer->Detach(v_sizer);
        delete v_sizer;

        // trigger re-layout
        this->right_v_sizer->Layout();

        // remove our record
        this->contact_widgets.erase(it);
    }
}
