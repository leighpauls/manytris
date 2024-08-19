constant constexpr size_t W = 10;
constant constexpr size_t H = 26;
constant constexpr size_t NUM_BLOCKS = W*H;
constant constexpr size_t FIELD_BYTES = NUM_BLOCKS / 8 + ((NUM_BLOCKS % 8) ? 1 : 0);

struct Field {
  uint8_t bytes[FIELD_BYTES];
};

struct TetrominoPositions {
  uint8_t pos[4][2];
};


struct DropConfig {
  uint32_t tetromino_idx;
  uint32_t initial_field_idx;
  uint32_t dest_field_idx;
  uint8_t left_shifts;
  uint8_t right_shifts;
};

struct FieldAddr {
  size_t byte_index;
  uint8_t mask;
};

FieldAddr addr(uint8_t x, uint8_t y) {
  size_t bit_index = y * W + x;
  size_t byte_index = bit_index / 8;
  size_t offset = bit_index % 8;
  uint8_t mask = 1 << offset;
  return FieldAddr {
    .byte_index = byte_index,
    .mask = mask,
  };
}


bool is_occupied(device Field* f, FieldAddr a) {
  return (f->bytes[a.byte_index] & a.mask) != 0;
}


void assign_pos(device Field* f, FieldAddr a, bool value) {
  if (value) {
    f->bytes[a.byte_index] |= a.mask;
  } else {
    f->bytes[a.byte_index] &= ~a.mask;
  }
}

void set_pos(device Field* f, FieldAddr a) {
  assign_pos(f, a, true);
}

enum ShiftDir {Down, Left, Right};

bool try_shift(device Field* f, thread TetrominoPositions* tp, ShiftDir d) {
  for (auto i = 0; i < 4; i++) {
    thread uint8_t* p = tp->pos[i];
    switch (d) {
    case Down:
      if (p[1] == 0 || is_occupied(f, addr(p[0], p[1]-1))) {
        return false;
      }
      break;
    case Left:
      if (p[0] == 0 || is_occupied(f, addr(p[0]-1, p[1]))) {
        return false;
      }
      break;
    case Right:
      if (p[0] == W-1 || is_occupied(f, addr(p[0]+1, p[1]))) {
        return false;
      }
      break;
    }
  }
  for (auto i = 0; i < 4; i++) {
    thread uint8_t* p = tp->pos[i];
    switch (d) {
    case Down:
      p[1] -= 1;
      break;
    case Left:
      p[0] -= 1;
      break;
    case Right:
      p[0] += 1;
      break;
    }
  }
  return true;
}

[[kernel]] void drop_tetromino(
    device const TetrominoPositions* tp,
    device Field* fields,
    device const DropConfig* configs,
    uint config_index [[thread_position_in_grid]]) {
  auto config = &configs[config_index];
  auto source_field = &fields[config->initial_field_idx];
  auto dest_field = &fields[config->dest_field_idx];

  *dest_field = *source_field;

  auto p = tp[config->tetromino_idx];

  // Shift left and right
  for (auto i = 0; i < config->left_shifts; i++) {
    try_shift(dest_field, &p, ShiftDir::Left);
  }
  for (auto i = 0; i < config->right_shifts; i++) {
    try_shift(dest_field, &p, ShiftDir::Right);
  }

  // Drop
  while (try_shift(dest_field, &p, ShiftDir::Down)) {}

  // Apply to the field
  for (size_t i = 0; i < 4; i++) {
    set_pos(dest_field, addr(p.pos[i][0], p.pos[i][1]));
  }

  // Look for lines
  auto drop_dist = 0;
  for (size_t y = 0; y < H; y++) {
    bool complete_line = true;
    for (size_t x = 0; x < W; x++) {
      auto a = addr(x, y);
      bool value = is_occupied(dest_field, a);
      if (!value) {
        complete_line = false;
      }

      auto dest_a = addr(x, y-drop_dist);
      assign_pos(dest_field, dest_a, value);

      // Explicitly clear the top lines which "fell" from above the field.
      if (y + drop_dist >= H) {
        assign_pos(dest_field, a, false);
      }
    }

    if (complete_line) {
      drop_dist += 1;
    }
  }
}
