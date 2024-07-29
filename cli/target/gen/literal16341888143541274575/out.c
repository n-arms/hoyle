
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

void literal(void *_result) {
  char _0[8];
  *(double *) _0 = 3;
  memmove(_result, _0, 8);
}
