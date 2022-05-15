/* For NumWorks n0110 calculators */
MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 176K + 16K
  QSPI : ORIGIN = 0x90000000, LENGTH = 8M
}

SECTIONS {
  .external : {
    (*(.external .external.*);
  } > QSPI
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

