# [BUG] Window border terdapat unnecessary space

## Description
Window Border pada feature dock terlalu tinggi, bahkan by default seperti ada space antara dock dan window app lainnya (user window space)

## Steps to Reproduce
1. Run nexora, enable dock
2. Change settings dock to be fixed

## Expected Behavior
Dock seharusnya tidak memiliki space antara window user space dan dock

## Actual Behavior
Terdapat unnecessary space antara window user space dan dock

## Evidence
[001-window-border](./evidence/001-window-border.png)

## Acceptance Criteria
- [ ] remove space between dock and  window user space