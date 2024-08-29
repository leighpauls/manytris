constant constexpr size_t W = 10;
constant constexpr size_t H = 26;
constant constexpr size_t NUM_BLOCKS = W*H;
constant constexpr size_t FIELD_BYTES = NUM_BLOCKS / 8 + ((NUM_BLOCKS % 8) ? 1 : 0);

constant constexpr size_t MAX_SEARCH_DEPTH = 6;
constant constexpr size_t ROTATIONS_PER_SHAPE = 4;
constant constexpr size_t SHIFTS_PER_ROTATION = 10;
constant constexpr uint32_t OUTPUTS_PER_INPUT_FIELD = static_cast<uint32_t>(ROTATIONS_PER_SHAPE * SHIFTS_PER_ROTATION);
constant constexpr size_t NUM_SHAPES = 7;

struct Field {
  uint8_t bytes[FIELD_BYTES];
};

struct TetrominoPositions {
  uint8_t pos[4][2];
};


struct DropConfig {
  uint32_t tetromino_idx;
  uint32_t next_tetromino_idx;
  uint32_t initial_field_idx;
  uint32_t dest_field_idx;
  uint8_t left_shifts;
  uint8_t right_shifts;
};


struct MoveResultScore {
    bool game_over;
    uint8_t lines_cleared;
    uint8_t height;
    uint16_t covered;
};


struct ShapeStartingPositions {
  TetrominoPositions bot_positions[4];
  TetrominoPositions player_position;
};

struct ShapePositionConfig {
  ShapeStartingPositions starting_positions[NUM_SHAPES];
};


struct SearchParams {
  uint8_t cur_search_depth;
  // List of the avilable upcoming shapes, starting with the first move of the search.
  uint8_t upcoming_shape_idxs[MAX_SEARCH_DEPTH + 1];
};


struct ComputedDropConfig {
  uint8_t shape_idx;
  uint8_t cw_rotations;
  uint32_t src_field_idx;
  uint32_t dest_field_idx;
  uint8_t left_shifts;
  uint8_t right_shifts;
  uint32_t thread_index;
  uint32_t input_start;
  uint32_t input_offset;
  uint32_t output_start;
  uint32_t depth;
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

uint32_t int_pow(uint32_t base, uint32_t exp) {
  uint32_t res = 1;
  for (uint32_t i = 0; i < exp; i++) {
    res *= base;
  }
  return res;
}

[[kernel]] void compute_drop_config(
    device const SearchParams* search_params,
    device ComputedDropConfig* drop_params,
    uint thread_idx [[thread_position_in_grid]]) {
  // if depth == 0, input fields are 0..1, output fields are 1..41
  // if depth == 1, input fields are 1..41, output fields are 41..(41+40*40)
  // if depth == 2, input fields are 41..(41+40*40), output fields are (41+40*40)..((41+40*40)+40*40*40)
  uint32_t input_field_start = 0;
  uint32_t output_field_start = 1;
  for (uint32_t i = 0; i < search_params->cur_search_depth; i++) {
    input_field_start += int_pow(OUTPUTS_PER_INPUT_FIELD, i);
    output_field_start += int_pow(OUTPUTS_PER_INPUT_FIELD, i+1);
  }

  uint32_t initial_field_idx = input_field_start + (static_cast<uint32_t>(thread_idx) / OUTPUTS_PER_INPUT_FIELD);
  uint32_t output_field_idx = output_field_start + thread_idx;

  // Order of moves for each input is:
  // (rot 0, shift 0), (rot 0, shift 1)..(rot 3, shift 9)
  uint32_t start_position_idx = thread_idx % OUTPUTS_PER_INPUT_FIELD;
  uint8_t num_rotations = static_cast<uint8_t>(start_position_idx / SHIFTS_PER_ROTATION);
  int32_t shifts = (start_position_idx % SHIFTS_PER_ROTATION) - 4;
  uint8_t right_shifts = (shifts > 0) ? static_cast<uint8_t>(shifts) : 0;
  uint8_t left_shifts = (shifts > 0) ? 0 : static_cast<uint8_t>(-shifts);

  drop_params[output_field_idx - 1] = ComputedDropConfig {
    .shape_idx = search_params->upcoming_shape_idxs[search_params->cur_search_depth],
    .cw_rotations = num_rotations,
    .src_field_idx = initial_field_idx,
    .dest_field_idx = output_field_idx,
    .left_shifts = left_shifts,
    .right_shifts = right_shifts,
    .thread_index = thread_idx,
    .input_start = input_field_start,
    .input_offset = (static_cast<uint32_t>(thread_idx) / OUTPUTS_PER_INPUT_FIELD),
    .output_start = output_field_start,
    .depth = search_params->cur_search_depth,
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

void do_drop_tetromino(
  TetrominoPositions p,
  TetrominoPositions next_p,
  device const Field* source_field,
  device Field* dest_field,
  device MoveResultScore* score,
  uint8_t left_shifts,
  uint8_t right_shifts);

[[kernel]] void drop_tetromino(
    device const TetrominoPositions* tp,
    device Field* fields,
    device const DropConfig* configs,
    device MoveResultScore* scores,
    uint config_index [[thread_position_in_grid]]) {
  auto config = &configs[config_index];
  auto source_field = &fields[config->initial_field_idx];
  auto dest_field = &fields[config->dest_field_idx];
  auto score = &scores[config_index];

  do_drop_tetromino(
    tp[config->tetromino_idx],
    tp[config->next_tetromino_idx],
    source_field,
    dest_field,
    score,
    config->left_shifts,
    config->right_shifts);
}

[[kernel]] void drop_tetromino_for_config(
  device const SearchParams* search_params,
  device const ShapePositionConfig* spc,
  device Field* fields,
  device const ComputedDropConfig* configs,
  device MoveResultScore* scores,
  uint thread_idx [[thread_position_in_grid]]
) {
  auto search_depth = search_params->cur_search_depth;
  uint32_t params_start = 0;
  for (uint32_t i = 0; i < search_depth; i++) {
    params_start += int_pow(OUTPUTS_PER_INPUT_FIELD, i+1);
  }

  auto config_idx = params_start + thread_idx;
  auto config = configs[config_idx];

  auto p = spc->starting_positions[search_params->upcoming_shape_idxs[search_depth]]
    .bot_positions[config.cw_rotations];
  auto next_p = spc->starting_positions[search_params->upcoming_shape_idxs[search_depth+1]]
    .player_position;

  device MoveResultScore* score = &scores[config_idx];

  do_drop_tetromino(
    p,
    next_p,
    &fields[config.src_field_idx],
    &fields[config.dest_field_idx],
    score,
    config.left_shifts,
    config.right_shifts);

  // Accumulate the previous config, if any
  if (search_params->cur_search_depth > 0) {
    auto prev_score_idx = config.src_field_idx - 1;
    auto prev_score = scores[prev_score_idx];
    if (prev_score.game_over) {
      *score = prev_score;
    } else {
      score->lines_cleared += prev_score.lines_cleared;
    }
  }
}

void do_drop_tetromino(
  TetrominoPositions p,
  TetrominoPositions next_p,
  device const Field* source_field,
  device Field* dest_field,
  device MoveResultScore* score,
  uint8_t left_shifts,
  uint8_t right_shifts) {

  *dest_field = *source_field;

  // Shift left and right
  for (auto i = 0; i < left_shifts; i++) {
    try_shift(dest_field, &p, ShiftDir::Left);
  }
  for (auto i = 0; i < right_shifts; i++) {
    try_shift(dest_field, &p, ShiftDir::Right);
  }

  // Drop
  while (try_shift(dest_field, &p, ShiftDir::Down)) {}

  // Apply to the field
  for (size_t i = 0; i < 4; i++) {
    set_pos(dest_field, addr(p.pos[i][0], p.pos[i][1]));
  }

  // Look for lines
  uint8_t lines_cleared = 0;
  uint8_t max_height = 0;
  for (size_t y = 0; y < H; y++) {
    bool complete_line = true;
    for (size_t x = 0; x < W; x++) {
      auto a = addr(x, y);
      bool value = is_occupied(dest_field, a);
      if (!value) {
        complete_line = false;
      } else {
        max_height = y+1;
      }

      auto dest_a = addr(x, y-lines_cleared);
      assign_pos(dest_field, dest_a, value);

      // Explicitly clear the top lines which "fell" from above the field.
      if (y + lines_cleared >= H) {
        assign_pos(dest_field, a, false);
      }
    }

    if (complete_line) {
      lines_cleared += 1;
    }
  }

  uint8_t final_height = max_height - lines_cleared;
  if (final_height > 22) {
    final_height = 22;
  }

  // Find the number of covered empty spaces
  uint16_t covered = 0;
  if (final_height > 1) {
    for (uint8_t x = 0; x < W; x++) {
      uint8_t column_top = final_height-1;
      while (column_top > 0) {
        if (is_occupied(dest_field, addr(x, column_top))) {
          break;
        }
        column_top -= 1;
      }
      for (uint8_t y = 0; y < column_top; y++) {
        if (!is_occupied(dest_field, addr(x, y))) {
          covered += 1;
        }
      }
    }
  }

  // See if this prevents the next tetromino from being placed.
  bool game_over = false;
  for (size_t i = 0; i < 4; i++) {
    if (is_occupied(dest_field, addr(next_p.pos[i][0], next_p.pos[i][1]))) {
      game_over = true;
      lines_cleared = 0;
    }
  }

  *score = MoveResultScore {
    .game_over = game_over,
    .lines_cleared = lines_cleared,
    .height = final_height,
    .covered = covered,
  };
}
