
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

void first_of(void *_result, void *x, void *y, void *a, void *b) {
  (((_witness *) b) -> destory)(y, ((_witness *) b) -> extra);
  (((_witness *) a) -> move)(_result, x, ((_witness *) a) -> extra);
}

void first(void *_result) {
  char _1[8];
  *(double *) _1 = 3;
  char _2[8];
  *(double *) _2 = 4;
  char _3[24];
  F64(_3);
  char _4[24];
  F64(_4);
  char _0[8];
  char _5[8];
  memmove(_5, _1, 8);
  char _6[8];
  memmove(_6, _2, 8);
  char _7[24];
  memmove(_7, _3, 24);
  char _8[24];
  memmove(_8, _4, 24);
  first_of(_0, _5, _6, _7, _8);
  memmove(_result, _0, 8);
}
