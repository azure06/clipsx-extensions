import { test } from 'node:test'
import assert from 'node:assert/strict'
import { attachTrackpadGestures } from './trackpad.mjs'
import { fitTransform } from './viewport.mjs'
test('WebKit pinch uses cumulative magnification without keyboard modifiers', () => {
 const target = new EventTarget()
 target.getBoundingClientRect = () => ({ left: 0, top: 0, width: 400, height: 400 })
 const content = { width: 1600, height: 800 }, viewport = { width: 400, height: 400 }
 let value = fitTransform(content, viewport), cancellations = 0
 const handler = attachTrackpadGestures(target, () => value, next => { value = next }, content, () => viewport, () => { cancellations++ })
 const send = (type, scale) => { const event = new Event(type, { cancelable: true }); Object.defineProperty(event, 'scale', { value: scale }); target.dispatchEvent(event); return event }
 send('gesturechange', 2)
 assert.equal(value.scale, .25)
 send('gesturestart', Number.NaN)
 assert.equal(handler.active(), false)
 assert.equal(send('gesturestart', 1).defaultPrevented, true)
 assert.equal(handler.active(), true)
 send('gesturechange', 1.5)
 assert.equal(value.scale, .375)
 send('gesturechange', 2)
 assert.deepEqual(value, { scale: .5, fit: false, x: -200, y: 0 })
 send('gesturechange', 0)
 assert.equal(value.scale, .5)
 assert.equal(send('gestureend', 2).defaultPrevented, true)
 assert.equal(handler.active(), false)
 send('gesturestart', 1)
 send('gesturechange', .5)
 assert.equal(value.scale, .25)
 assert.equal(cancellations, 2)
 handler.dispose()
 send('gesturechange', 2)
 assert.equal(value.scale, .25)
})
