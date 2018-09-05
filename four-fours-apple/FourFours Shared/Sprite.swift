//
//  Sprite.swift
//  FourFours
//
//  Created by Kevin Guthrie on 8/23/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class Sprite : SKSpriteNode {
  
  var dragHandlers: [DragHandler] = []
  
  var currentTexture: Texture?
  var eventSink: Sprite?
  
  init() {
    super.init(
        texture: nil,
        color: SKColor.clear,
        size: CGSize(width: 0, height: 0))
    anchorPoint = CGPoint(x: 0.0, y: 1.0)
    self.isHidden = true
  }
  
  required init?(coder aDecoder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }
  
  func setVisible(_ visible: Bool) {
    DispatchQueue.main.async {
      self.isHidden = !visible
    }
  }
  
  func setTexture(_ texture: Texture) {
    DispatchQueue.main.async {
      self.currentTexture = texture
      self.texture = texture.texture
      self.size = texture.texture.size()
    }
  }
  
  func setSizeAnimated(_ width: Float64, _ height: Float64, _ durationSeconds: Float64) {
    run(SKAction.resize(
        toWidth: CGFloat(width),
        height: CGFloat(height),
        duration: durationSeconds))
  }
  
  func setLocationAnimated(_ left: Float64, _ top: Float64, _ durationSeconds: Float64) {
    run(SKAction.move(to: CGPoint(x: CGFloat(left), y: -CGFloat(top)),
                  duration: durationSeconds))
    
  }
  
  func addDragHandler(_ handler: DragHandler) -> HandlerRegistration {
    DispatchQueue.main.sync {
      self.isUserInteractionEnabled = true
      self.dragHandlers.append(handler)
    }
    return HandlerRegistration(deregister_callback: {
      self.removeHandler(handler)
    })
  }
  
  func removeHandler(_ handler: DragHandler) {
    DispatchQueue.main.sync {
      if let index = self.dragHandlers.index(of: handler) {
        self.dragHandlers.remove(at: index)
      }
    }
  }
  
  func propagateEventsTo(_ sprite: Sprite) {
    DispatchQueue.main.sync {
      self.isUserInteractionEnabled = true
      self.eventSink = sprite
    }
  }
  
  deinit {
    print("Dropping Sprite")
  }
}

extension Sprite {
  
  #if os(iOS) || os(tvOS)
  override func touchesStart(_ touches: Set<UITouch>, with event: UIEvent?) {
    DispatchQueue.main.async {
      self.dragHandlers.forEach { (handler) in
        handler.onDragStart(x: Float64(event.absoluteX), y: Float64(event.absoluteY))
      }
    }
  }
  #endif
  
  #if os(OSX)
  
  override func mouseDown(with event: NSEvent) {
    DispatchQueue.main.async {
      self.eventSink?.mouseDown(with: event)
      
      let localPoint = event.location(in: self)
      
      self.dragHandlers.forEach { (handler) in
        handler.onDragStart(
            globalX: Float64(event.locationInWindow.x),
            globalY: Float64((event.window?.contentView?.bounds.size.height)!
                - event.locationInWindow.y),
            localX: Float64(localPoint.x),
            localY: -Float64(localPoint.y))
      }
    }
  }
  
  override func mouseDragged(with event: NSEvent) {
    DispatchQueue.main.async {
      self.eventSink?.mouseDragged(with: event)
      let localPoint = event.location(in: self)
      self.dragHandlers.forEach { (handler) in
        handler.onDragMove(
          globalX: Float64(event.locationInWindow.x),
          globalY: Float64((event.window?.contentView?.bounds.size.height)!
            - event.locationInWindow.y),
          localX: Float64(localPoint.x),
          localY: -Float64(localPoint.y))
      }
    }
  }
  
  override func mouseUp(with event: NSEvent) {
    DispatchQueue.main.async {
      self.eventSink?.mouseUp(with: event)
      let localPoint = event.location(in: self)
      self.dragHandlers.forEach { (handler) in
        handler.onDragEnd(
          globalX: Float64(event.locationInWindow.x),
          globalY: Float64((event.window?.contentView?.bounds.size.height)!
            - event.locationInWindow.y),
          localX: Float64(localPoint.x),
          localY: -Float64(localPoint.y))
      }
    }
  }
  #endif
}
