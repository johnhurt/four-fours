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
  
  var currentTexture: Texture?
  
  init() {
    super.init(
        texture: nil,
        color: SKColor.clear,
        size: CGSize(width: 0, height: 0))
    anchorPoint = CGPoint(x: 0.0, y: 1.0)
  }
  
  required init?(coder aDecoder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }
  
  func setTexture(_ texture: Texture) {
    DispatchQueue.main.async {
      self.currentTexture = texture
      self.texture = texture.texture
      self.size = texture.texture.size()
    }
  }
  
  func setSizeAnimated(_ width: Int64, _ height: Int64, _ durationSeconds: Float64) {
    run(SKAction.resize(
        toWidth: CGFloat(width),
        height: CGFloat(height),
        duration: durationSeconds))
  }
  
  func setLocationAnimated(_ left: Int64, _ top: Int64, _ durationSeconds: Float64) {
    
    run(SKAction.move(to: CGPoint(x: CGFloat(left), y: -CGFloat(top)),
                  duration: durationSeconds))
    
  }
}
