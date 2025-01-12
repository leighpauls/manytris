#extension GL_EXT_shader_8bit_storage : enable
#extension GL_EXT_shader_explicit_arithmetic_types : enable


const uint MAX_SEARCH_DEPTH = 6;
const uint ROTATIONS_PER_SHAPE = 4;
const uint SHIFTS_PER_ROTATION = 10;
const uint32_t OUTPUTS_PER_INPUT_FIELD = ROTATIONS_PER_SHAPE * SHIFTS_PER_ROTATION;

struct ComputedDropConfig {
  uint8_t shape_idx;
  uint8_t cw_rotations;
  uint32_t src_field_idx;
  uint32_t dest_field_idx;
  uint8_t left_shifts;
  uint8_t right_shifts;
};

struct SearchParams {
    uint8_t cur_search_depth;
    uint8_t upcoming_shape_idxs[MAX_SEARCH_DEPTH + 1];
};


uint32_t int_pow(uint32_t base, uint32_t exp) {
  uint32_t res = 1;
  for (uint32_t i = 0; i < exp; i++) {
    res *= base;
  }
  return res;
}
