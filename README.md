### Small console program for switching PulseAudio outputs and adjusting their volume.

Use Up/Down to select an output, Left/Right to change its volume in 5% steps,
Enter to switch to the selected output, and x to exit. The volume shown at
startup is read from PulseAudio for active ports. The level of an inactive port
is unknown until it is selected. PulseAudio's module-device-restore keeps the
per-port levels; pacc does not create a settings file. Changes to an inactive
port are applied when you select it with Enter.
