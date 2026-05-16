#!/usr/bin/env perl

use strict;
use warnings;

my $target = q{};
my $width  = 24;

while (my $line = <>) {
  if ($line =~ m{^([-_[:alnum:]]+):}) {
    $target = $1;
  }

  next unless $line =~ s{.*\$\(call Test,}{};

  # Remove trailing right parenthesis
  $line =~ s{[)]}{}g;
  chomp $line;

  $line = sprintf('| %-*s | Test %s', $width, $target, $line);

  print "$line\n";
}