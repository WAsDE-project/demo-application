__attribute__((import_module("host")))
void ModifyTextView(void* textview_handle, const char* text);

static char* messages[] = {
    "You already got my attention.",
    "Stop pressing that.",
    "Are you listening to me? Stop."
};

#define LENGTH (sizeof(messages)/sizeof(*messages))

static int index = 0;

__attribute__((used))
int write_message(void* textview_handle) {
    if (index == 0) {
        ModifyTextView(textview_handle, "Hello there!");
        ++index;
    } else {
        int msg_index = index++ % LENGTH;
        ModifyTextView(textview_handle, messages[msg_index]);
    }
    return 1;
}
