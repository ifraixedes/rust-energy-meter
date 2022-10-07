# Electricity meter

Tiny command-line application to read "your electricity consumption" CSV data file downloaded from
[e-distribución][1] to sum them up according to the 3 day time periods.

## For what is useful?

Unfortunately, some energy companies, like [Holaluz][2], requires to you sending the meter reading
of the 3 time periods every month in order of charging your exact consumption rather than an
estimation, yes, that's pretty bad because they are many others that they care of reading that data
for you every month and always charge you your exact consumption.

The [e-distribution portal][3] portal give you access to the current counter, however, it gives you the
total sum but not the 3 time period sums. Yes, you can read those number directly form the meter,
but in some situations is cumbersome due to the meter is located or you're lazy to have to
physically and check them every month.

What solutions do you have?

1. Change energy company for one that do it for you and they never charge any estimated consumption.
1. Use this command-line application ;D

## How does it work?

The application read "you consumption" CSV data file that you can download from [e-distribution
portal][3]
and calculate the total sum for each of the 3 time periods.

Because you can read from one of the past energy company bills what were the total sum for each
period at certain date or download it from the "reading and consumption certificate" from
[e-distribution portal][3], then you can download "your consumption" from that date to the last day
that you want and sum up to the previous informed and get the total for each time period that you
can submit it to the energy company to get billed exactly for what you have consumed.

This is what this tiny command-line application does.


[1]: https://www.edistribucion.com/
[2]: https://www.holaluz.com/
[3]: https://zonaprivada.edistribucion.com/
