#pragma once
#include "enums.hpp"

struct ContactGroupHeadingPanel;
struct ContactPanel;

class ContactListPanel: public wxScrolled<wxControl> {
public:
    ContactListPanel(wxWindow* parent, tego_session_handle session_handle);

    void add_contact(
        const tego_user_handle user_handle,
        const wxString& nickname,
        const wxBitmap& avatar,
        const ContactGroup contact_group
    );
    void remove_contact(tego_user_handle contact_handle);

private:
    // event handlers
    void on_char(wxKeyEvent&);

    // setters/getters

    void
    set_selected_contact_group_heading_panel(ContactGroupHeadingPanel* contact_group_heading_panel);
    void set_selected_contact_panel(ContactPanel* contact_panel);

    void remove_contact_panel(ContactPanel* contact_panel);

    void set_group_expanded(ContactGroup contact_group, bool expanded);
    bool get_group_expanded(ContactGroup contact_group) const;

    // navigation methods
    void navigate_up();
    void navigate_down();
    void navigate_out();
    void navigate_in();

    // event emiitters
    void emit_contact_selected(std::optional<tego_user_handle> contact_handle);
    void emit_contact_removed(tego_user_handle contact_handle);

    // parent group nodes for each of our contact groups
    ContactGroupHeadingPanel* group_heading_panel[static_cast<size_t>(ContactGroup::Count)];
    // box-sizers containing each of the contacts within each group
    wxBoxSizer* group_v_sizer[static_cast<size_t>(ContactGroup::Count)];

    // pointers to currently selected item in list (group or contact)
    ContactGroupHeadingPanel* selected_contact_group_heading_panel = nullptr;
    ContactPanel* selected_contact_panel = nullptr;

    std::unordered_map<tego_user_handle, ContactPanel*> contact_map;
};
