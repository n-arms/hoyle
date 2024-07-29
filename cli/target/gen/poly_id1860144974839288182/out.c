
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

void poly_id(void *_result) {
  char _1[8];
  *(double *) _1 = 3;
  char _2[24];
  F64(_2);
  char _0[8];
  char _3[8];
  memmove(_3, _1, 8);
  char _4[24];
  memmove(_4, _2, 24);
  id(_0, _3, _4);
  memmove(_result, _0, 8);
}
