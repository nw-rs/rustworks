/* For NumWorks n0110 calculators */
MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x90000000, LENGTH = 8M
  RAM : ORIGIN = 0x20000000, LENGTH = 176K + 16K
}
