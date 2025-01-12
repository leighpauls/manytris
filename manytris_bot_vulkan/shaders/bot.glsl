#version 460
#extension GL_EXT_shader_8bit_storage : enable
#extension GL_EXT_shader_explicit_arithmetic_types : enable

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

const uint MAX_SEARCH_DEPTH = 6;
const uint ROTATIONS_PER_SHAPE = 4;
const uint SHIFTS_PER_ROTATION = 10;
const uint32_t OUTPUTS_PER_INPUT_FIELD = ROTATIONS_PER_SHAPE * SHIFTS_PER_ROTATION;

layout(set = 0, binding = 0) buffer SearchParams {
    uint8_t cur_search_depth;
    uint8_t upcoming_shape_idxs[MAX_SEARCH_DEPTH + 1];
} search_params;

struct ComputedDropConfig {
  uint8_t shape_idx;
  uint8_t cw_rotations;
  uint32_t src_field_idx;
  uint32_t dest_field_idx;
  uint8_t left_shifts;
  uint8_t right_shifts;
};

layout(set = 0, binding = 1) buffer ComputedDropConfigs {
    ComputedDropConfig configs[];
} drop_configs;


uint32_t int_pow(uint32_t base, uint32_t exp);

void main() {
    uint32_t thread_idx = gl_GlobalInvocationID.x;
    if (thread_idx >= drop_configs.configs.length()) {
      return;
    }

    // if depth == 0, input fields are 0..1, output fields are 1..41
    // if depth == 1, input fields are 1..41, output fields are 41..(41+40*40)
    // if depth == 2, input fields are 41..(41+40*40), output fields are (41+40*40)..((41+40*40)+40*40*40)
    uint32_t src_field_start = 0;
    uint32_t dest_field_start = 1;
    for (uint32_t i = 0; i < search_params.cur_search_depth; i++) {
        src_field_start += int_pow(OUTPUTS_PER_INPUT_FIELD, i);
        dest_field_start += int_pow(OUTPUTS_PER_INPUT_FIELD, i+1);
    }

    uint32_t src_field_idx = src_field_start + (thread_idx / OUTPUTS_PER_INPUT_FIELD);
    uint32_t dest_field_idx = dest_field_start + thread_idx;

    // Order of moves for each input is:
    // (rot 0, shift 0), (rot 0, shift 1)..(rot 3, shift 9)
    uint8_t shape_idx = search_params.upcoming_shape_idxs[search_params.cur_search_depth];
    uint32_t start_position_idx = thread_idx % OUTPUTS_PER_INPUT_FIELD;
    uint8_t num_rotations = uint8_t(start_position_idx / SHIFTS_PER_ROTATION);
    int32_t shifts = int32_t(start_position_idx % SHIFTS_PER_ROTATION) - 4;
    uint8_t right_shifts = (shifts > 0) ? uint8_t(shifts) : uint8_t(0);
    uint8_t left_shifts = (shifts > 0) ? uint8_t(0) : uint8_t(-shifts);

    drop_configs.configs[thread_idx] = ComputedDropConfig(
        shape_idx,
        num_rotations,
        src_field_idx,
        dest_field_idx,
        left_shifts,
        right_shifts
    );
}

uint32_t int_pow(uint32_t base, uint32_t exp) {
  uint32_t res = 1;
  for (uint32_t i = 0; i < exp; i++) {
    res *= base;
  }
  return res;
}
