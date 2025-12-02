#!/usr/sbin/dtrace -s

#pragma D option quiet

dtrace:::BEGIN
{
    minor_faults = 0;
    major_faults = 0;
}

vminfo:::maj_fault
/pid == $target/
{
    major_faults++;
}

vminfo:::as_fault
/pid == $target/
{
    minor_faults++;
}

dtrace:::END
{
    printf("PAGEFAULT STATS:\n");
    printf("major faults: %d\n", major_faults);
    printf("minor faults: %d\n", minor_faults);
}