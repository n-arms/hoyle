
#include <string.h>
typedef struct _witness {
  void (*move)(void *, void *, void *);
  void *extra;
} _witness;

void move_F64(void *dest, void *src, void *extra) {
  memmove(dest, src, 8);
}

void F64(void *_result) {
  _witness *result = _result;
  result -> move = move_F64;
  result -> extra = NULL;
}

void id(void *_result, void *x, void *t) {
  (((_witness *) t) -> move)(_result, x, ((_witness *) t) -> extra);
}

void literal(void *_result) {
  char _0[8];
  *(double *) _0 = 3;
  memmove(_result, _0, 8);
}

void chained_poly_id_literal(void *_result) {
  char _2[8];
  literal(_2);
  char _3[24];
  F64(_3);
  char _1[8];
  char _5[8];
  memmove(_5, _2, 8);
  char _6[24];
  memmove(_6, _3, 24);
  id(_1, _5, _6);
  char _4[24];
  F64(_4);
  char _0[8];
  char _7[8];
  memmove(_7, _1, 8);
  char _8[24];
  memmove(_8, _4, 24);
  id(_0, _7, _8);
  memmove(_result, _0, 8);
}
