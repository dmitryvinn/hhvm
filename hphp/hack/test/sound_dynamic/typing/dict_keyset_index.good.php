<?hh
function test(dynamic $y):void {
  $y as nonnull;
  $x = keyset[];
  $d = dict[];
  $x[] = $y;
  $d[$y] = 3;
  hh_expect_equivalent<keyset<arraykey>>($x);
  hh_expect_equivalent<dict<arraykey, int>>($d);
}
