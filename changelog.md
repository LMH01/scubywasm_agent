# Changelog

## v1.0.8

- slightly reduce own ship size when calculating if shot would hit. `ctx.config.ship_hit_radius + (ctx.config.ship_hit_radius + 0.25)` -> `ctx.config.ship_hit_radius + (ctx.config.ship_hit_radius + 0.125)`

## v1.0.7

- slightly increase own ship size when calculating if shot would hit to make evasion easier

## v1.0.6

- turn straight when no shot available and not evading to not fly into enemies anymore

## v1.0.5

- don't evade when taking shot

## v1.0.4

- revert changes in v1.0.2

## v1.0.3

- fix aiming to actually shoot nearest target

## v1.0.2

- try to evade other ships

## v1.0.1

- only fire shot when distance is <= 0.3

## v1.0.0

initial release
