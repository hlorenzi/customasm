#ruledef {
  a {a} => {
    assert(a == 0)
    0`1
  }
  a {a} => {
    1`1 @ asm {
      a ({a}-1)
    }
  }
}
a 3 ; error: failed / note:_:6: within / note:_:8: a (3-1) / error:_:8: failed / note:_:8: a ((((((((((((3-1)-1)-1)-1)-1)-1)-1)-1)-1)-1)-1)-1) / error:_:7: recursion