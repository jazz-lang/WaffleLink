void *get_stack_top()
{
    return __builtin_frame_address(0);
}