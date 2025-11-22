use crate::keyexpr;

fn intersect(left: &str, right: &str) -> bool {
    let left = keyexpr::new(left).unwrap();
    let right = keyexpr::new(right).unwrap();

    left.intersects(right)
}

#[test]
fn keyexpr_intersect() {
    assert!(intersect("a", "a"));
    assert!(intersect("a/b", "a/b"));
    assert!(intersect("*", "abc"));
    assert!(intersect("*", "xxx"));
    assert!(intersect("ab$*", "abcd"));
    assert!(intersect("ab$*d", "abcd"));
    assert!(intersect("ab$*", "ab"));
    assert!(!intersect("ab/*", "ab"));
    assert!(intersect("a/*/c/*/e", "a/b/c/d/e"));
    assert!(intersect("a/$*b/c/$*d/e", "a/xb/c/xd/e"));
    assert!(!intersect("a/*/c/*/e", "a/c/e"));
    assert!(!intersect("a/*/c/*/e", "a/b/c/d/x/e"));
    assert!(!intersect("ab$*cd", "abxxcxxd"));
    assert!(intersect("ab$*cd", "abxxcxxcd"));
    assert!(!intersect("ab$*cd", "abxxcxxcdx"));
    assert!(intersect("**", "abc"));
    assert!(intersect("**", "a/b/c"));
    assert!(intersect("ab/**", "ab"));
    assert!(intersect("**/xyz", "a/b/xyz/d/e/f/xyz"));
    assert!(!intersect("**/xyz$*xyz", "a/b/xyz/d/e/f/xyz"));
    assert!(intersect("**/xyz$*xyz", "a/b/xyzdefxyz"));
    assert!(intersect("a/**/c/**/e", "a/b/b/b/c/d/d/d/e"));
    assert!(intersect("a/**/c/**/e", "a/c/e"));
    assert!(intersect("a/**/c/*/e/*", "a/b/b/b/c/d/d/c/d/e/f"));
    assert!(!intersect("a/**/c/*/e/*", "a/b/b/b/c/d/d/c/d/d/e/f"));
    assert!(!intersect("ab$*cd", "abxxcxxcdx"));
    assert!(intersect("x/abc", "x/abc"));
    assert!(!intersect("x/abc", "abc"));
    assert!(intersect("x/*", "x/abc"));
    assert!(!intersect("x/*", "abc"));
    assert!(!intersect("*", "x/abc"));
    assert!(intersect("x/*", "x/abc$*"));
    assert!(intersect("x/$*abc", "x/abc$*"));
    assert!(intersect("x/a$*", "x/abc$*"));
    assert!(intersect("x/a$*de", "x/abc$*de"));
    assert!(intersect("x/a$*d$*e", "x/a$*e"));
    assert!(intersect("x/a$*d$*e", "x/a$*c$*e"));
    assert!(intersect("x/a$*d$*e", "x/ade"));
    assert!(!intersect("x/c$*", "x/abc$*"));
    assert!(!intersect("x/$*d", "x/$*e"));

    assert!(intersect("@a", "@a"));
    assert!(!intersect("@a", "@ab"));
    assert!(!intersect("@a", "@a/b"));
    assert!(!intersect("@a", "@a/*"));
    assert!(!intersect("@a", "@a/*/**"));
    assert!(!intersect("@a", "@a$*/**"));
    assert!(intersect("@a", "@a/**"));
    assert!(!intersect("**/xyz$*xyz", "@a/b/xyzdefxyz"));
    assert!(intersect("@a/**/c/**/e", "@a/b/b/b/c/d/d/d/e"));
    assert!(!intersect("@a/**/c/**/e", "@a/@b/b/b/c/d/d/d/e"));
    assert!(intersect("@a/**/@c/**/e", "@a/b/b/b/@c/d/d/d/e"));
    assert!(intersect("@a/**/e", "@a/b/b/d/d/d/e"));
    assert!(intersect("@a/**/e", "@a/b/b/b/d/d/d/e"));
    assert!(intersect("@a/**/e", "@a/b/b/c/d/d/d/e"));
    assert!(!intersect("@a/**/e", "@a/b/b/@c/b/d/d/d/e"));
    assert!(!intersect("@a/*", "@a/@b"));
    assert!(!intersect("@a/**", "@a/@b"));
    assert!(intersect("@a/**/@b", "@a/@b"));
    assert!(intersect("@a/@b/**", "@a/@b"));
    assert!(intersect("@a/**/@c/**/@b", "@a/**/@c/@b"));
    assert!(intersect("@a/**/@c/**/@b", "@a/@c/**/@b"));
    assert!(intersect("@a/**/@c/@b", "@a/@c/**/@b"));
    assert!(!intersect("@a/**/@b", "@a/**/@c/**/@b"));
}
