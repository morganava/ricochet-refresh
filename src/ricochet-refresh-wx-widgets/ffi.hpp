#pragma once

std::unique_ptr<tego_string> into_tego_string(const wxString& wx_string);
wxString into_wxString(const std::unique_ptr<tego_string>& value);
wxString into_wxString(const tego_string* value);