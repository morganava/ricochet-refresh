#pragma once

std::unique_ptr<tego_string> into_tego_string(const wxString& value);
wxString into_wxString(const std::unique_ptr<tego_string>& value);
wxString into_wxString(const tego_string* value);
tego_time into_tego_time(const wxDateTime& value);
wxDateTime into_wxDateTime(const tego_time value);
