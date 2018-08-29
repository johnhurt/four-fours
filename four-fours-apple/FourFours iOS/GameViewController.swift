//
//  GameViewController.swift
//  FourFours iOS
//
//  Created by Kevin Guthrie on 8/9/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import UIKit
import SpriteKit
import GameplayKit

class GameViewController: UIViewController {

  var skView : SKView?
  var scene : GameScene?

  override func viewDidLoad() {
    super.viewDidLoad()
  

    // Present the scene
    self.skView = self.view as? SKView
    
    self.scene = GameScene.newGameScene(size: (self.skView?.bounds.size)!)
    
    self.skView?.presentScene(self.scene)
    
    self.skView?.ignoresSiblingOrder = true
    self.skView?.showsFPS = true
    self.skView?.showsNodeCount = true
    
  }

  override var shouldAutorotate: Bool {
    return true
  }

  override var supportedInterfaceOrientations: UIInterfaceOrientationMask {
    if UIDevice.current.userInterfaceIdiom == .phone {
      return .allButUpsideDown
    } else {
      return .all
    }
  }

  override func didReceiveMemoryWarning() {
    super.didReceiveMemoryWarning()
    // Release any cached data, images, etc that aren't in use.
  }

  override var prefersStatusBarHidden: Bool {
    return true
  }
  
  override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
    super.viewWillTransition(to: size, with: coordinator)
    
    self.scene?.setSize(size: size)
  }
}
