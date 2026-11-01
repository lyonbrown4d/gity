export const CI_CONFIG = `pipeline {
  name = "perf"
}

stage smoke {
  tags = ["linux"]
  run {
    shell("echo perf")
  }
}
`;
