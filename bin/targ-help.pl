#!/usr/bin/env perl

use strict;
use warnings;

my $width = 24;

while (my $line = <>) {
  next unless $line =~ s{^#[|]}{};
  chomp $line;

  if ($line =~ m{^\s*[*]\s+([-_[:alnum:]]+)\s+-\s+(.*)$}) {
    my ($target, $description) = ($1, $2);
    $line = sprintf('| %-*s | %s', $width, $target, $description);
  }
  elsif ($line =~ m{^\s*\|\s*(.*?)\s*\|\s*(.*)$}) {
    my ($target, $description) = ($1, $2);
    $line = sprintf('| %-*s | %s', $width, $target, $description);
  }

  print "$line\n";
}