- [ ] Tighten up the check whether stdin and/or stderr is a TTY. Right now we
      only check stdin, and so in pipes - even if we have stderr == TTY - we
      don't interactively prompt. We probably should, or at least if we detect
      that the job is large. Or even better may be whether any TTY is available
      at all, without having to go through stdin/stdout/stderr.
- [ ] Investigate whether Run Length Encoding (RLE) would be worthwhile
      complexity for deeply repetitive output. I'm thinking no, since most
      workloads are "wide" instead of "deep". Same vein, validate the data
      model's `task_lines` sequence number rebuild strategy is worthwhile.
