#include "out.c"
#include <stdio.h>

int main() {
  double x;
  mono_id(&x);
  printf("%lf", x);
}