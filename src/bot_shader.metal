[[kernel]] void test_func(
    device const uint32_t* buffer,
    device const uint32_t* items_per_thread_buffer,
    device uint32_t* output,
    uint gid [[thread_position_in_grid]]) {
  size_t items_per_thread = *items_per_thread_buffer;
  size_t start_idx = gid * items_per_thread;
  size_t end_idx = start_idx + items_per_thread;
  uint32_t result = 0;
  for (size_t i = start_idx; i < end_idx; i++) {
    result += buffer[i];
  }
  output[gid] = result;
}

