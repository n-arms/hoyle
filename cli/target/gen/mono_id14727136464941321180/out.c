
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

void id(void *_result, void *x) {
  memmove(_result, x, 8);
}

void mono_id(void *_result) {
  char _1[8];
  *(double *) _1 = 3;
  char _0[8];
  char _2[8];
  memmove(_2, _1, 8);
  id(_0, _2);
  memmove(_result, _0, 8);
}
