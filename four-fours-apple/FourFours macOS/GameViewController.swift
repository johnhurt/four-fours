//
//  GameViewController.swift
//  FourFours macOS
//
//  Created by Kevin Guthrie on 8/9/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Cocoa
import SpriteKit
import GameplayKit

class GameViewController: NSViewController {

  var skView : SKView?
  var scene : GameScene?

  override func viewDidLoad() {
    super.viewDidLoad()
  
    // Present the scene
    self.skView = self.view as? SKView
  
    self.scene = GameScene.newGameScene(size: (skView?.bounds.size)!)
    self.skView!.presentScene(self.scene)
  
    self.skView!.ignoresSiblingOrder = true
  
    self.skView!.showsFPS = true
    self.skView!.showsNodeCount = true
  }
  
  override func viewDidLayout() {
    super.viewDidLayout()
    
    self.scene?.setSize(size: self.skView!.bounds.size)
  }
  
}

